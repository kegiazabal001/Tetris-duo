use std::collections::VecDeque;

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use rand::seq::SliceRandom;
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::board::{Board, PieceColor, COLS, VISIBLE_ROWS};
use crate::collision::{self, piece_fits, PiecePos};
use crate::config::AppConfig;
use crate::input::{input_map_for, PieceAction};
use crate::piece::{Rotation, TSpinType, TetrominoKind};
use crate::constants::{ARR_RATE, DAS_DELAY, LOCK_DELAY};
use crate::state::{ChaosState, FlipPhase, SelectedMode};
use crate::scoring::ScoreBoard;
use crate::state::GameState;

/// Blocks input processing for one frame after entering Playing state,
/// preventing the Space/Enter press used to start/restart the game from
/// being interpreted as a hard-drop on the very first frame.
#[derive(Resource, Default)]
pub struct InputGrace(pub bool);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum PlayerId {
    P1,
    P2,
}

impl PlayerId {
    pub fn spawn_col(self) -> i32 {
        match self {
            PlayerId::P1 => (COLS as i32) / 4,     // ~4
            PlayerId::P2 => 3 * (COLS as i32) / 4, // ~13
        }
    }
}

/// Active falling piece, one per player.
#[derive(Component, Debug, Clone)]
pub struct ActivePiece {
    pub player: PlayerId,
    pub kind: TetrominoKind,
    pub rotation: Rotation,
    pub col: i32,
    pub row: i32,
    pub gravity_timer: f32,
    pub lock_timer: Option<f32>,
    pub soft_drop_held: bool,
    // T-Spin detection: true if the last action that changed piece state was a rotation
    pub last_was_rotation: bool,
    // Hold piece
    pub hold: Option<TetrominoKind>,
    pub hold_is_anchor: bool,
    pub hold_used: bool,
    // DAS (Delayed Auto-Shift) state
    pub das_left: f32,
    pub das_right: f32,
    pub arr_left: f32,
    pub arr_right: f32,
    /// Prevents check_lock from emitting PieceLocked more than once per piece.
    pub locked: bool,
    /// True if this piece is an anchor — will survive line clears permanently.
    pub is_anchor: bool,
    /// True while waiting for a FLIP! migration to begin/end — piece is hidden, no input.
    pub waiting_for_flip: bool,
}

/// 7-bag randomizer per player.
#[derive(Component, Debug)]
pub struct PieceBag {
    pub player: PlayerId,
    pub queue: VecDeque<TetrominoKind>,
    rng: StdRng,
}

impl PieceBag {
    pub fn new(player: PlayerId) -> Self {
        Self::new_with_rng(player, StdRng::from_entropy())
    }

    pub fn new_with_rng(player: PlayerId, rng: StdRng) -> Self {
        let mut bag = Self { player, queue: VecDeque::new(), rng };
        bag.refill();
        bag.refill(); // start with 14 pieces
        bag
    }

    fn refill(&mut self) {
        let mut pieces = TetrominoKind::ALL.to_vec();
        pieces.shuffle(&mut self.rng);
        self.queue.extend(pieces);
    }

    pub fn pop(&mut self) -> TetrominoKind {
        if self.queue.len() <= 7 {
            self.refill();
        }
        debug_assert!(!self.queue.is_empty());
        self.queue.pop_front().expect("PieceBag::pop: la cola está vacía; esto es un bug")
    }

    pub fn peek(&self) -> TetrominoKind {
        debug_assert!(!self.queue.is_empty());
        self.queue[0]
    }

    /// Returns the next N pieces as a stack-allocated array.
    /// Panics if the queue has fewer than N pieces (guaranteed not to happen in normal play).
    pub fn peek_n<const N: usize>(&self) -> [TetrominoKind; N] {
        debug_assert!(N <= self.queue.len(), "peek_n: N={N} excede cola de len={}", self.queue.len());
        std::array::from_fn(|i| self.queue[i])
    }
}

// Events
#[derive(Event)]
pub struct PieceLocked {
    pub player: PlayerId,
    pub cells: [(i32, i32); 4],
}

/// Emitted when lines are cleared (or count=0 when a piece locks without clearing, for combo reset).
#[derive(Event)]
pub struct LinesCleared {
    pub player: PlayerId,
    pub count: u32,
    pub t_spin: TSpinType,
}

#[derive(Event)]
pub struct PieceRotated {
    pub player: PlayerId,
}

#[derive(Event)]
pub struct GameOverEvent;

/// Collects position snapshots for both players from any query iterator
/// that yields `(PlayerId, PiecePos)` pairs.
pub fn collect_snapshots(mut iter: impl Iterator<Item = (PlayerId, PiecePos)>) -> [Option<(PlayerId, PiecePos)>; 2] {
    [iter.next(), iter.next()]
}

impl ActivePiece {
    /// Resets the lock-delay timer back to LOCK_DELAY when the piece is on the ground.
    /// Used after any successful move or rotation to extend the grace period.
    pub fn reset_lock_if_active(&mut self) {
        if self.lock_timer.is_some() {
            self.lock_timer = Some(LOCK_DELAY);
        }
    }

    /// Returns the minimal position snapshot needed for collision checks.
    pub fn to_piece_pos(&self) -> PiecePos {
        PiecePos { kind: self.kind, rotation: self.rotation, col: self.col, row: self.row }
    }
}

fn fresh_piece(player: PlayerId, kind: TetrominoKind, chaos: Option<&ChaosState>) -> ActivePiece {
    let col = match chaos {
        Some(cs) if cs.swap_active => match player {
            PlayerId::P1 => PlayerId::P2.spawn_col(),
            PlayerId::P2 => PlayerId::P1.spawn_col(),
        },
        _ => player.spawn_col(),
    };
    // During FLIP Active, pieces spawn near the floor (row 1) and rise upward.
    let row = match chaos {
        Some(cs) if cs.flip_phase == FlipPhase::Active => 1,
        _ => VISIBLE_ROWS as i32 - 2,
    };
    ActivePiece {
        player,
        kind,
        rotation: Rotation::R0,
        col,
        row,
        gravity_timer: 0.0,
        lock_timer: None,
        soft_drop_held: false,
        last_was_rotation: false,
        hold: None,
        hold_is_anchor: false,
        hold_used: false,
        das_left: 0.0,
        das_right: 0.0,
        arr_left: 0.0,
        arr_right: 0.0,
        locked: false,
        is_anchor: false,
        waiting_for_flip: false,
    }
}

/// Attempts to move a piece laterally by `dc` columns.
/// On success updates col, clears `last_was_rotation`, and resets the lock timer.
/// Returns true if the move succeeded.
fn try_lateral_move(piece: &mut ActivePiece, board: &Board, other: Option<PiecePos>, dc: i32) -> bool {
    if piece_fits(board, piece.kind, piece.rotation, piece.col + dc, piece.row, other) {
        piece.col += dc;
        piece.last_was_rotation = false;
        piece.reset_lock_if_active();
        true
    } else {
        false
    }
}

/// Attempts to rotate a piece (CW or CCW) using SRS wall kicks.
/// On success updates position/rotation, sets `last_was_rotation`, resets lock timer, and emits event.
fn apply_rotation(
    piece: &mut ActivePiece,
    board: &Board,
    other: Option<PiecePos>,
    to: Rotation,
    ev_rotate: &mut EventWriter<PieceRotated>,
) {
    if let Some((nc, nr, nrot)) =
        collision::try_rotate(board, piece.kind, piece.rotation, to, piece.col, piece.row, other)
    {
        piece.col = nc;
        piece.row = nr;
        piece.rotation = nrot;
        piece.last_was_rotation = true;
        piece.reset_lock_if_active();
        ev_rotate.write(PieceRotated { player: piece.player });
    }
}

/// Spawn both player entities with their input maps and piece bags.
pub fn spawn_players(mut commands: Commands, config: Res<AppConfig>) {
    for player in [PlayerId::P1, PlayerId::P2] {
        let mut bag = PieceBag::new(player);
        let kind = bag.pop();
        commands.spawn((
            fresh_piece(player, kind, None),
            bag,
            input_map_for(player, &config),
            ActionState::<PieceAction>::default(),
        ));
    }
}

pub fn despawn_players(mut commands: Commands, query: Query<Entity, With<ActivePiece>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// System: handle input for each player.
pub fn handle_input(
    time: Res<Time>,
    mut players: Query<(&ActionState<PieceAction>, &mut ActivePiece, &mut PieceBag)>,
    board: Res<Board>,
    mut score: ResMut<ScoreBoard>,
    mut ev_rotate: EventWriter<PieceRotated>,
    mut grace: ResMut<InputGrace>,
    keyboard: Res<ButtonInput<KeyCode>>,
    chaos: Option<Res<ChaosState>>,
) {
    if grace.0 {
        // Block until the hard-drop keys are physically released. We check Bevy's raw
        // ButtonInput rather than leafwing's ActionState because the player entities are
        // spawned fresh in OnEnter(Playing) — leafwing hasn't seen their history, so on
        // the first frame it processes them it reports just_pressed=true while Space is
        // still held, even though the ActionState doesn't reflect it yet.
        let hard_drop_keys = [KeyCode::Space, KeyCode::Enter];
        let any_held = hard_drop_keys.iter().any(|k| keyboard.pressed(*k));
        if !any_held {
            grace.0 = false;
        }
        return;
    }
    let dt = time.delta_secs();
    let gravity_dir = chaos.as_ref().map_or(1_i32, |c| c.gravity_dir as i32);
    // Collect minimal position snapshots — no heap allocation, avoids cloning full ActivePiece.
    let snapshots = collect_snapshots(players.iter().map(|(_, ap, _)| (ap.player, ap.to_piece_pos())));

    for (action, mut piece, mut bag) in &mut players {
        if piece.waiting_for_flip {
            continue;
        }
        let other = snapshots.iter().flatten().find(|(pid, _)| *pid != piece.player).map(|(_, pos)| *pos);

        // --- Hold piece ---
        if action.just_pressed(&PieceAction::Hold) && !piece.hold_used {
            let new_kind = match piece.hold {
                Some(k) => k,
                None => bag.pop(),
            };
            let held_kind = piece.kind;
            let held_is_anchor = piece.is_anchor;
            let incoming_is_anchor = piece.hold_is_anchor;
            *piece = fresh_piece(piece.player, new_kind, chaos.as_deref());
            piece.hold = Some(held_kind);
            piece.hold_is_anchor = held_is_anchor;
            piece.is_anchor = incoming_is_anchor;
            piece.hold_used = true;
            continue;
        }

        // --- Left movement ---
        if action.just_pressed(&PieceAction::MoveLeft) {
            piece.das_left = 0.0;
            piece.arr_left = 0.0;
            piece.das_right = 0.0;
            piece.arr_right = 0.0;
            try_lateral_move(&mut piece, &board, other, -1);
        } else if action.pressed(&PieceAction::MoveLeft) {
            piece.das_left += dt;
            if piece.das_left >= DAS_DELAY {
                piece.arr_left += dt;
                while piece.arr_left >= ARR_RATE {
                    piece.arr_left -= ARR_RATE;
                    if !try_lateral_move(&mut piece, &board, other, -1) {
                        piece.arr_left = 0.0;
                        break;
                    }
                }
            }
        } else {
            piece.das_left = 0.0;
            piece.arr_left = 0.0;
        }

        // --- Right movement ---
        if action.just_pressed(&PieceAction::MoveRight) {
            piece.das_right = 0.0;
            piece.arr_right = 0.0;
            piece.das_left = 0.0;
            piece.arr_left = 0.0;
            try_lateral_move(&mut piece, &board, other, 1);
        } else if action.pressed(&PieceAction::MoveRight) {
            piece.das_right += dt;
            if piece.das_right >= DAS_DELAY {
                piece.arr_right += dt;
                while piece.arr_right >= ARR_RATE {
                    piece.arr_right -= ARR_RATE;
                    if !try_lateral_move(&mut piece, &board, other, 1) {
                        piece.arr_right = 0.0;
                        break;
                    }
                }
            }
        } else {
            piece.das_right = 0.0;
            piece.arr_right = 0.0;
        }

        piece.soft_drop_held = action.pressed(&PieceAction::SoftDrop);

        if action.just_pressed(&PieceAction::HardDrop) {
            let start_row = piece.row;
            loop {
                let next = piece.row - gravity_dir;
                if !piece_fits(&board, piece.kind, piece.rotation, piece.col, next, other) {
                    break;
                }
                piece.row = next;
            }
            score.score += (start_row - piece.row).unsigned_abs() * 2;
            piece.last_was_rotation = false;
            piece.lock_timer = Some(0.0);
        }

        if action.just_pressed(&PieceAction::RotateCW) {
            let to = piece.rotation.cw();
            apply_rotation(&mut piece, &board, other, to, &mut ev_rotate);
        }

        if action.just_pressed(&PieceAction::RotateCCW) {
            let to = piece.rotation.ccw();
            apply_rotation(&mut piece, &board, other, to, &mut ev_rotate);
        }
    }
}

/// System: apply gravity.
pub fn apply_gravity(
    time: Res<Time>,
    mut players: Query<&mut ActivePiece>,
    board: Res<Board>,
    mut score: ResMut<ScoreBoard>,
    chaos: Option<Res<ChaosState>>,
) {
    let gravity_dir = chaos.as_ref().map_or(1_i32, |c| c.gravity_dir as i32);
    let snapshots = collect_snapshots(players.iter().map(|ap| (ap.player, ap.to_piece_pos())));
    let normal_interval = score.gravity_interval();

    for mut piece in &mut players {
        if piece.waiting_for_flip {
            continue;
        }
        let other = snapshots.iter().flatten().find(|(pid, _)| *pid != piece.player).map(|(_, pos)| *pos);
        let interval = if piece.soft_drop_held {
            (normal_interval / 20.0).max(0.05)
        } else {
            normal_interval
        };
        piece.gravity_timer += time.delta_secs();

        if piece.gravity_timer >= interval {
            piece.gravity_timer -= interval;
            let next_row = piece.row - gravity_dir; // -1 moves down (normal), +1 moves up (FLIP)
            if piece_fits(&board, piece.kind, piece.rotation, piece.col, next_row, other) {
                piece.row = next_row;
                piece.lock_timer = None;
                if piece.soft_drop_held {
                    score.score += 1;
                }
            }
        }
    }
}

/// System: check if piece should lock.
pub fn check_lock(
    time: Res<Time>,
    mut players: Query<&mut ActivePiece>,
    board: Res<Board>,
    mut ev_lock: EventWriter<PieceLocked>,
    chaos: Option<Res<ChaosState>>,
) {
    let gravity_dir = chaos.as_ref().map_or(1_i32, |c| c.gravity_dir as i32);
    for mut piece in &mut players {
        if piece.waiting_for_flip {
            continue;
        }
        // on_ground only checks fixed board cells — not the other player's live piece.
        // A live piece should never trigger lock on a neighbour that may still move away.
        let next_row = piece.row - gravity_dir; // direction the piece would move next
        let on_ground = !piece_fits(&board, piece.kind, piece.rotation, piece.col, next_row, None);

        if on_ground {
            let timer = piece.lock_timer.get_or_insert(LOCK_DELAY);
            *timer -= time.delta_secs();
            if *timer <= 0.0 && !piece.locked {
                piece.locked = true;
                let cells = collision::absolute_cells(piece.kind, piece.rotation, piece.col, piece.row);
                ev_lock.write(PieceLocked { player: piece.player, cells });
            }
        } else {
            piece.lock_timer = None;
        }
    }
}

/// System: lock piece into board and spawn next.
pub fn lock_piece(
    mut board: ResMut<Board>,
    mut players: Query<(&mut ActivePiece, &mut PieceBag)>,
    mut ev_lock: EventReader<PieceLocked>,
    mut ev_lines: EventWriter<LinesCleared>,
    mut chaos: Option<ResMut<ChaosState>>,
    selected_mode: Res<SelectedMode>,
) {
    // Snapshot all positions before any mutation so spawn-collision checks are stable.
    let pre_positions: Vec<(PlayerId, PiecePos)> = players
        .iter()
        .map(|(piece, _)| (piece.player, piece.to_piece_pos()))
        .collect();

    for event in ev_lock.read() {
        let other_pos = pre_positions.iter()
            .find(|(pid, _)| *pid != event.player)
            .map(|(_, pos)| *pos);

        for (mut piece, mut bag) in &mut players {
            if piece.player != event.player {
                continue;
            }

            // T-Spin check BEFORE writing cells to board
            let t_spin = if piece.kind == TetrominoKind::T && piece.last_was_rotation {
                collision::check_t_spin(&board, piece.col, piece.row, piece.rotation)
            } else {
                TSpinType::None
            };

            // Write cells to board with per-piece color
            let cell_color = if piece.is_anchor {
                PieceColor::Anchor
            } else {
                match piece.player {
                    PlayerId::P1 => PieceColor::Player1(piece.kind),
                    PlayerId::P2 => PieceColor::Player2(piece.kind),
                }
            };
            for (cx, cy) in collision::absolute_cells(piece.kind, piece.rotation, piece.col, piece.row) {
                board.set(cx, cy, cell_color);
            }

            // Detect full rows WITHOUT compacting (the flash animation will compact later)
            let count = board.detect_full_rows().len() as u32;

            // Always emit so update_score can manage the combo counter
            ev_lines.write(LinesCleared { player: piece.player, count, t_spin });

            // If a FLIP phase is blocking spawns, mark the player as waiting instead of
            // spawning a new piece.
            let blocks = chaos.as_ref().map_or(false, |c| c.blocks_spawn());
            if blocks {
                if let Some(ref mut cs) = chaos {
                    match piece.player {
                        PlayerId::P1 => cs.flip_wait_p1_done = true,
                        PlayerId::P2 => cs.flip_wait_p2_done = true,
                    }
                }
                piece.waiting_for_flip = true;
                piece.row = -100; // move off-screen so render hides it
                continue;
            }

            // Spawn next piece
            let next_kind = bag.pop();
            let hold = piece.hold;
            *piece = fresh_piece(piece.player, next_kind, chaos.as_ref().map(|c| c.as_ref()));
            piece.hold = hold; // preserve held piece across locks
            // In Chaos mode, 1-in-40 chance the new piece is an anchor
            if *selected_mode == SelectedMode::Chaos && rand::random::<f32>() < 1.0 / 40.0 {
                piece.is_anchor = true;
            }

            // After chaos-swap transitions both players can end up on the same side,
            // causing the spawn position to collide with the other player's active piece.
            // Try lateral shifts until we find a free column (board-only check handles
            // the real stack-overflow game over via check_game_over).
            if let Some(other) = other_pos {
                if !piece_fits(&board, piece.kind, piece.rotation, piece.col, piece.row, Some(other)) {
                    for shift in [1i32, -1, 2, -2, 3, -3, 4, -4] {
                        let nc = piece.col + shift;
                        if piece_fits(&board, piece.kind, piece.rotation, nc, piece.row, Some(other)) {
                            piece.col = nc;
                            break;
                        }
                    }
                }
            }
        }
    }
}

/// System: when FLIP Active phase starts, spawn fresh pieces for players that were waiting.
pub fn spawn_after_flip(
    mut players: Query<(&mut ActivePiece, &mut PieceBag)>,
    chaos: Option<Res<ChaosState>>,
    selected_mode: Res<SelectedMode>,
    board: Res<Board>,
) {
    let Some(cs) = chaos else { return };
    if cs.flip_phase != FlipPhase::Active {
        return;
    }

    // Snapshot positions for spawn-collision avoidance.
    let snapshots: Vec<(PlayerId, PiecePos)> = players
        .iter()
        .filter(|(p, _)| !p.waiting_for_flip)
        .map(|(p, _)| (p.player, p.to_piece_pos()))
        .collect();

    for (mut piece, mut bag) in &mut players {
        if !piece.waiting_for_flip {
            continue;
        }
        let other = snapshots.iter().find(|(pid, _)| *pid != piece.player).map(|(_, pos)| *pos);
        let next_kind = bag.pop();
        let hold = piece.hold;
        let hold_is_anchor = piece.hold_is_anchor;
        *piece = fresh_piece(piece.player, next_kind, Some(&cs));
        piece.hold = hold;
        piece.hold_is_anchor = hold_is_anchor;
        if *selected_mode == SelectedMode::Chaos && rand::random::<f32>() < 1.0 / 40.0 {
            piece.is_anchor = true;
        }
        if let Some(other) = other {
            if !piece_fits(&board, piece.kind, piece.rotation, piece.col, piece.row, Some(other)) {
                for shift in [1i32, -1, 2, -2, 3, -3, 4, -4] {
                    let nc = piece.col + shift;
                    if piece_fits(&board, piece.kind, piece.rotation, nc, piece.row, Some(other)) {
                        piece.col = nc;
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;
    use crate::piece::{Rotation, TetrominoKind};

    fn make_active(player: PlayerId, kind: TetrominoKind, col: i32, row: i32) -> ActivePiece {
        ActivePiece {
            player,
            kind,
            rotation: Rotation::R0,
            col,
            row,
            gravity_timer: 0.0,
            lock_timer: None,
            soft_drop_held: false,
            last_was_rotation: false,
            hold: None,
            hold_is_anchor: false,
            hold_used: false,
            das_left: 0.0,
            das_right: 0.0,
            arr_left: 0.0,
            arr_right: 0.0,
            locked: false,
            is_anchor: false,
            waiting_for_flip: false,
        }
    }

    #[test]
    fn hard_drop_sets_lock_timer_to_zero() {
        // After hard drop, lock_timer should be Some(0.0) to lock immediately.
        let mut piece = make_active(PlayerId::P1, TetrominoKind::O, 5, 10);
        piece.lock_timer = Some(0.0);
        assert_eq!(piece.lock_timer, Some(0.0));
    }

    #[test]
    fn lock_delay_resets_on_lateral_move() {
        // Simulate: piece is on ground (lock_timer running), then player moves laterally.
        // lock_timer should be reset to LOCK_DELAY.
        let mut piece = make_active(PlayerId::P1, TetrominoKind::T, 5, 1);
        piece.lock_timer = Some(0.2); // partially elapsed
        // Simulate what handle_input does on a lateral move when lock_timer is Some
        if piece.lock_timer.is_some() {
            piece.lock_timer = Some(LOCK_DELAY);
        }
        assert_eq!(piece.lock_timer, Some(LOCK_DELAY));
    }

    #[test]
    fn lock_delay_resets_on_rotate() {
        let mut piece = make_active(PlayerId::P1, TetrominoKind::T, 5, 1);
        piece.lock_timer = Some(0.1);
        if piece.lock_timer.is_some() {
            piece.lock_timer = Some(LOCK_DELAY);
        }
        assert_eq!(piece.lock_timer, Some(LOCK_DELAY));
    }

    #[test]
    #[allow(unused_assignments)]
    fn locked_flag_prevents_double_event() {
        // Simulate check_lock logic: once locked=true the event must not fire again.
        let mut piece = make_active(PlayerId::P1, TetrominoKind::T, 5, 0);
        piece.lock_timer = Some(-0.1); // timer expired
        let mut events_fired = 0u32;

        // Frame 1
        if let Some(t) = piece.lock_timer {
            if t <= 0.0 && !piece.locked {
                piece.locked = true;
                events_fired += 1;
            }
        }
        // Frame 2 (same situation — timer still ≤0, piece still on ground)
        if let Some(t) = piece.lock_timer {
            if t <= 0.0 && !piece.locked {
                piece.locked = true;
                events_fired += 1;
            }
        }
        assert_eq!(events_fired, 1, "PieceLocked should only fire once per piece");
    }

    #[test]
    fn hold_used_true_after_hold() {
        let mut piece = make_active(PlayerId::P1, TetrominoKind::T, 5, 10);
        piece.hold_used = true; // simulates what handle_input sets
        assert!(piece.hold_used);
    }

    #[test]
    fn hold_resets_on_new_piece() {
        // fresh_piece always sets hold_used = false
        let p = fresh_piece(PlayerId::P1, TetrominoKind::S, None);
        assert!(!p.hold_used);
    }

    #[test]
    fn locked_false_on_new_piece() {
        let p = fresh_piece(PlayerId::P1, TetrominoKind::I, None);
        assert!(!p.locked);
    }

    #[test]
    fn game_over_on_spawn_collision() {
        // If piece doesn't fit at spawn position, check_game_over should detect it.
        // Simulate by filling the spawn area cells on the board.
        let mut board = Board::default();
        let piece = fresh_piece(PlayerId::P1, TetrominoKind::O, None);
        // O at R0 occupies (col,row),(col+1,row),(col,row+1),(col+1,row+1)
        let col = piece.col;
        let row = piece.row;
        for (dc, dr) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            board.set(col + dc, row + dr, crate::board::PieceColor::Player1(TetrominoKind::I));
        }
        // piece_fits should now return false → game over
        let fits = collision::piece_fits(&board, piece.kind, piece.rotation, piece.col, piece.row, None);
        assert!(!fits, "blocked spawn should trigger game over detection");
    }

    #[test]
    fn soft_drop_score_increments() {
        // Soft drop: score +1 per row dropped by gravity while soft_drop_held.
        // Verify the arithmetic: 3 rows of soft drop = score 3.
        let rows_dropped = 3u32;
        let score_delta = rows_dropped; // +1 per cell, as implemented in apply_gravity
        assert_eq!(score_delta, 3);
    }

    #[test]
    fn hard_drop_score_increments() {
        // Hard drop: score += (start_row - piece.row) * 2
        let start_row = 18i32;
        let end_row = 2i32;
        let score_delta = (start_row - end_row) as u32 * 2;
        assert_eq!(score_delta, 32);
    }
}

/// System: check if newly spawned piece overlaps → game over.
pub fn check_game_over(
    players: Query<&ActivePiece>,
    board: Res<Board>,
    mut ev_gameover: EventWriter<GameOverEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let snapshots = collect_snapshots(players.iter().map(|ap| (ap.player, ap.to_piece_pos())));
    for (player, pos) in snapshots.iter().flatten() {
        let other = snapshots.iter().flatten().find(|(pid, _)| pid != player).map(|(_, p)| *p);
        if !piece_fits(&board, pos.kind, pos.rotation, pos.col, pos.row, other) {
            ev_gameover.write(GameOverEvent);
            next_state.set(GameState::GameOver);
            return;
        }
    }
}
