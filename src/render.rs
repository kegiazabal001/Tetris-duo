use bevy::prelude::*;

use crate::board::{Board, PieceColor, COLS, VISIBLE_ROWS};
use crate::collision::{self, piece_fits};
use crate::piece::{self, Rotation, TetrominoKind};
use crate::piece::TSpinType;
use crate::collision::PiecePos;
use crate::player::{ActivePiece, LinesCleared, PieceBag, PieceLocked, PlayerId};
use crate::scoring::{LevelUpEvent, ScoreBoard};

pub use crate::constants::CELL_SIZE;
pub const BOARD_OFFSET_X: f32 = -(COLS as f32 * CELL_SIZE) / 2.0;
pub const BOARD_OFFSET_Y: f32 = -(VISIBLE_ROWS as f32 * CELL_SIZE) / 2.0;

const PREVIEW_CELL_SIZE: f32 = 20.0;
const P1_PANEL_X: f32 = -306.0;
const P2_PANEL_X: f32 = 306.0;
const NEXT_PREVIEW_Y: f32 = 170.0;
const HOLD_PREVIEW_Y: f32 = -80.0;
const NEXT_PREVIEW_COUNT: usize = 3;
const NEXT_PREVIEW_SLOT_H: f32 = 70.0;

pub use crate::constants::{LINE_CLEAR_FLASH_DURATION, LOCK_FLASH_DURATION};

// ── Color helpers ─────────────────────────────────────────────────────────────

/// Tetris Guideline vivid colors (P1).
fn kind_color_vivid(kind: TetrominoKind) -> Color {
    match kind {
        TetrominoKind::I => Color::srgb(0.0, 0.87, 0.87),  // cyan
        TetrominoKind::O => Color::srgb(0.93, 0.87, 0.0),  // yellow
        TetrominoKind::T => Color::srgb(0.60, 0.0, 0.87),  // purple
        TetrominoKind::S => Color::srgb(0.0, 0.87, 0.0),   // green
        TetrominoKind::Z => Color::srgb(0.87, 0.0, 0.0),   // red
        TetrominoKind::J => Color::srgb(0.0, 0.20, 0.87),  // blue
        TetrominoKind::L => Color::srgb(0.93, 0.60, 0.0),  // orange
    }
}

/// Pastel versions of the Guideline colors (P2): lerp each vivid color 40% toward white.
fn kind_color_pastel(kind: TetrominoKind) -> Color {
    let v = kind_color_vivid(kind).to_srgba();
    Color::srgb(
        v.red * 0.6 + 0.4,
        v.green * 0.6 + 0.4,
        v.blue * 0.6 + 0.4,
    )
}

fn color_for(pc: PieceColor) -> Color {
    match pc {
        PieceColor::Player1(k) => kind_color_vivid(k),
        PieceColor::Player2(k) => kind_color_pastel(k),
    }
}

fn active_color(player: PlayerId, kind: TetrominoKind) -> Color {
    match player {
        PlayerId::P1 => kind_color_vivid(kind),
        PlayerId::P2 => kind_color_pastel(kind),
    }
}

fn ghost_color(player: PlayerId, kind: TetrominoKind) -> Color {
    let base = active_color(player, kind).to_srgba();
    Color::srgba(base.red, base.green, base.blue, 0.22)
}

// ── Resources ─────────────────────────────────────────────────────────────────

/// Controls the line-clear flash + delayed board compaction.
#[derive(Resource, Default)]
pub struct LineClearFlash {
    pub timer: f32,
    pub pending_rows: Vec<usize>,
}

/// Brief white flash on the cells of a just-locked piece.
#[derive(Resource, Default)]
pub struct PieceLockFlash {
    pub timer: f32,
    pub cells: Vec<(usize, usize)>,
}

// ── Components ────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct PopupText {
    pub lifetime: f32,
    pub total: f32,
}

#[derive(Component)]
pub struct BoardBackdrop;

#[derive(Component)]
pub struct BoardCellSprite {
    pub col: usize,
    pub row: usize,
}

#[derive(Component)]
pub struct ActiveBlockSprite {
    pub player: PlayerId,
    pub index: usize,
}

#[derive(Component)]
pub struct GhostBlockSprite {
    pub player: PlayerId,
    pub index: usize,
}

#[derive(Component)]
pub struct NextPieceBlock {
    pub player: PlayerId,
    pub slot: usize,   // 0 = siguiente, 1 = +1, 2 = +2
    pub index: usize,  // 0..4 (bloque dentro de la pieza)
}

#[derive(Component)]
pub struct HoldPieceBlock {
    pub player: PlayerId,
    pub index: usize,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn cell_pos(col: usize, row: usize) -> Vec3 {
    Vec3::new(
        BOARD_OFFSET_X + col as f32 * CELL_SIZE + CELL_SIZE / 2.0,
        BOARD_OFFSET_Y + row as f32 * CELL_SIZE + CELL_SIZE / 2.0,
        0.0,
    )
}

/// Centering offset so each piece looks centered in its 4×4 preview box.
fn preview_center_offset(kind: TetrominoKind) -> (f32, f32) {
    let cells = piece::cells(kind, Rotation::R0);
    debug_assert_eq!(cells.len(), 4, "preview_center_offset: pieza no tiene 4 células");
    let n = cells.len() as f32;
    let cx: f32 = cells.iter().map(|(x, _)| *x as f32).sum::<f32>() / n;
    let cy: f32 = cells.iter().map(|(_, y)| *y as f32).sum::<f32>() / n;
    (-cx, -cy)
}

fn preview_block_pos(panel_x: f32, panel_y: f32, kind: TetrominoKind, index: usize) -> Vec3 {
    let cells = piece::cells(kind, Rotation::R0);
    let (ox, oy) = preview_center_offset(kind);
    let (dx, dy) = cells[index];
    Vec3::new(
        panel_x + (dx as f32 + ox + 0.5) * PREVIEW_CELL_SIZE,
        panel_y + (dy as f32 + oy + 0.5) * PREVIEW_CELL_SIZE,
        3.0,
    )
}

// ── Setup ─────────────────────────────────────────────────────────────────────

pub fn setup_board_visuals(mut commands: Commands) {
    let board_w = COLS as f32 * CELL_SIZE;
    let board_h = VISIBLE_ROWS as f32 * CELL_SIZE;

    // Outer border
    commands.spawn((
        Sprite {
            color: Color::srgb(0.38, 0.38, 0.50),
            custom_size: Some(Vec2::new(board_w + 8.0, board_h + 8.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, -2.0)),
        BoardBackdrop,
    ));

    // Inner background
    commands.spawn((
        Sprite {
            color: Color::srgb(0.06, 0.06, 0.09),
            custom_size: Some(Vec2::new(board_w, board_h)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
        BoardBackdrop,
    ));

    // Board cell grid
    for row in 0..VISIBLE_ROWS {
        for col in 0..COLS {
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.12, 0.12, 0.15),
                    custom_size: Some(Vec2::splat(CELL_SIZE - 2.0)),
                    ..default()
                },
                Transform::from_translation(cell_pos(col, row)),
                BoardCellSprite { col, row },
            ));
        }
    }

    // Active block sprites (4 per player)
    for player in [PlayerId::P1, PlayerId::P2] {
        for index in 0..4 {
            commands.spawn((
                Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::splat(CELL_SIZE - 2.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, -1000.0, 2.0)),
                ActiveBlockSprite { player, index },
            ));
        }
        for index in 0..4 {
            commands.spawn((
                Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::splat(CELL_SIZE - 2.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, -1000.0, 1.0)),
                GhostBlockSprite { player, index },
            ));
        }
    }

    // Panel backgrounds for P1 (left) and P2 (right)
    for (panel_x, _) in [(P1_PANEL_X, "P1"), (P2_PANEL_X, "P2")] {
        // Next preview background (covers 3 slots)
        let next_panel_center_y = NEXT_PREVIEW_Y - NEXT_PREVIEW_SLOT_H;
        commands.spawn((
            Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, 0.05),
                custom_size: Some(Vec2::new(88.0, 230.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(panel_x, next_panel_center_y, -0.5)),
            BoardBackdrop,
        ));
        // Hold preview background
        commands.spawn((
            Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, 0.05),
                custom_size: Some(Vec2::new(88.0, 88.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(panel_x, HOLD_PREVIEW_Y, -0.5)),
            BoardBackdrop,
        ));
    }

    // Next and Hold preview block sprites
    for player in [PlayerId::P1, PlayerId::P2] {
        // 3 slots × 4 blocks = 12 NextPieceBlock per player
        for slot in 0..NEXT_PREVIEW_COUNT {
            for index in 0..4 {
                commands.spawn((
                    Sprite {
                        color: Color::NONE,
                        custom_size: Some(Vec2::splat(PREVIEW_CELL_SIZE - 2.0)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(0.0, -1000.0, 3.0)),
                    NextPieceBlock { player, slot, index },
                ));
            }
        }
        for index in 0..4 {
            commands.spawn((
                Sprite {
                    color: Color::NONE,
                    custom_size: Some(Vec2::splat(PREVIEW_CELL_SIZE - 2.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, -1000.0, 3.0)),
                HoldPieceBlock { player, index },
            ));
        }
    }
}

pub fn despawn_board_visuals(
    mut commands: Commands,
    q1: Query<Entity, With<BoardCellSprite>>,
    q2: Query<Entity, With<ActiveBlockSprite>>,
    q3: Query<Entity, With<GhostBlockSprite>>,
    q4: Query<Entity, With<BoardBackdrop>>,
    q5: Query<Entity, With<NextPieceBlock>>,
    q6: Query<Entity, With<HoldPieceBlock>>,
    q7: Query<Entity, With<PopupText>>,
) {
    for e in q1.iter()
        .chain(q2.iter())
        .chain(q3.iter())
        .chain(q4.iter())
        .chain(q5.iter())
        .chain(q6.iter())
        .chain(q7.iter())
    {
        commands.entity(e).despawn();
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Sync board cell colors; flashes pending-clear rows and recently locked cells.
pub fn sync_board_cells(
    board: Res<Board>,
    flash: Res<LineClearFlash>,
    lock_flash: Res<PieceLockFlash>,
    mut query: Query<(&BoardCellSprite, &mut Sprite)>,
) {
    let flash_t = if flash.timer > 0.0 {
        flash.timer / LINE_CLEAR_FLASH_DURATION
    } else {
        0.0
    };
    // Pulse effect: 2 full cycles during the flash duration
    let pulse = if flash_t > 0.0 {
        (flash_t * std::f32::consts::PI * 4.0).sin().abs()
    } else {
        0.0
    };

    let lock_t = if lock_flash.timer > 0.0 {
        lock_flash.timer / LOCK_FLASH_DURATION
    } else {
        0.0
    };

    for (cell, mut sprite) in &mut query {
        let base = match board.cells[cell.row][cell.col] {
            Some(pc) => color_for(pc),
            None => Color::srgb(0.12, 0.12, 0.15),
        };
        let after_line_flash = if pulse > 0.0 && flash.pending_rows.contains(&cell.row) {
            let srgba = base.to_srgba();
            Color::srgba(
                srgba.red + (1.0 - srgba.red) * pulse,
                srgba.green + (1.0 - srgba.green) * pulse,
                srgba.blue + (1.0 - srgba.blue) * pulse,
                srgba.alpha,
            )
        } else {
            base
        };
        sprite.color = if lock_t > 0.0 && lock_flash.cells.contains(&(cell.col, cell.row)) {
            let srgba = after_line_flash.to_srgba();
            Color::srgba(
                srgba.red + (1.0 - srgba.red) * lock_t,
                srgba.green + (1.0 - srgba.green) * lock_t,
                srgba.blue + (1.0 - srgba.blue) * lock_t,
                srgba.alpha,
            )
        } else {
            after_line_flash
        };
    }
}

/// Start the flash animation when lines are cleared.
pub fn on_lines_cleared(
    mut ev: EventReader<LinesCleared>,
    mut flash: ResMut<LineClearFlash>,
    board: Res<Board>,
) {
    for event in ev.read() {
        if event.count > 0 {
            flash.pending_rows = board.detect_full_rows();
            flash.timer = LINE_CLEAR_FLASH_DURATION;
        }
    }
}

/// Tick flash timer; compact board when animation finishes.
pub fn tick_flash_timer(
    time: Res<Time>,
    mut flash: ResMut<LineClearFlash>,
    mut board: ResMut<Board>,
) {
    if flash.timer > 0.0 {
        flash.timer = (flash.timer - time.delta_secs()).max(0.0);
        if flash.timer == 0.0 && !flash.pending_rows.is_empty() {
            board.remove_rows(&flash.pending_rows);
            flash.pending_rows.clear();
        }
    }
}

/// Sync active piece sprites.
pub fn sync_active_pieces(
    players: Query<&ActivePiece>,
    mut blocks: Query<(&ActiveBlockSprite, &mut Transform, &mut Sprite)>,
) {
    for (block, mut tf, mut sprite) in &mut blocks {
        let Some(piece) = players.iter().find(|p| p.player == block.player) else {
            tf.translation.y = -1000.0;
            continue;
        };
        let cells = collision::absolute_cells(piece.kind, piece.rotation, piece.col, piece.row);
        let (cx, cy) = cells[block.index];
        if cy < VISIBLE_ROWS as i32 && cy >= 0 && cx >= 0 && cx < COLS as i32 {
            tf.translation = cell_pos(cx as usize, cy as usize);
            tf.translation.z = 2.0;
            sprite.color = active_color(piece.player, piece.kind);
        } else {
            tf.translation.y = -1000.0;
        }
    }
}

/// Sync ghost piece sprites.
pub fn sync_ghost_pieces(
    players: Query<&ActivePiece>,
    board: Res<Board>,
    mut ghosts: Query<(&GhostBlockSprite, &mut Transform, &mut Sprite)>,
) {
    // Collect minimal snapshots — no heap allocation, no full ActivePiece clone.
    let snapshots: [Option<(PlayerId, PiecePos)>; 2] = {
        let mut it = players.iter();
        [
            it.next().map(|ap| (ap.player, ap.to_piece_pos())),
            it.next().map(|ap| (ap.player, ap.to_piece_pos())),
        ]
    };

    for (ghost, mut tf, mut sprite) in &mut ghosts {
        let Some((player, pos)) = snapshots.iter().flatten().find(|(pid, _)| *pid == ghost.player) else {
            tf.translation.y = -1000.0;
            continue;
        };
        let other = snapshots.iter().flatten().find(|(pid, _)| *pid != ghost.player).map(|(_, p)| *p);

        let mut ghost_row = pos.row;
        while piece_fits(&board, pos.kind, pos.rotation, pos.col, ghost_row - 1, other) {
            ghost_row -= 1;
        }

        let cells = collision::absolute_cells(pos.kind, pos.rotation, pos.col, ghost_row);
        let (cx, cy) = cells[ghost.index];
        if cy < VISIBLE_ROWS as i32 && cy >= 0 && cx >= 0 && cx < COLS as i32 {
            tf.translation = cell_pos(cx as usize, cy as usize);
            tf.translation.z = 1.0;
            sprite.color = ghost_color(*player, pos.kind);
        } else {
            tf.translation.y = -1000.0;
        }
    }
}

/// Sync next-piece and hold-piece preview sprites.
pub fn sync_preview_pieces(
    players: Query<(&ActivePiece, &PieceBag)>,
    mut next_blocks: Query<
        (&NextPieceBlock, &mut Transform, &mut Sprite),
        Without<HoldPieceBlock>,
    >,
    mut hold_blocks: Query<
        (&HoldPieceBlock, &mut Transform, &mut Sprite),
        Without<NextPieceBlock>,
    >,
) {
    for (block, mut tf, mut sprite) in &mut next_blocks {
        let panel_x = if block.player == PlayerId::P1 { P1_PANEL_X } else { P2_PANEL_X };
        let Some((piece, bag)) = players.iter().find(|(p, _)| p.player == block.player) else {
            tf.translation.y = -1000.0;
            continue;
        };
        let preview = bag.peek_n::<NEXT_PREVIEW_COUNT>();
        if block.slot >= preview.len() {
            tf.translation.y = -1000.0;
            sprite.color = Color::NONE;
            continue;
        }
        let kind = preview[block.slot];
        let slot_y = NEXT_PREVIEW_Y - block.slot as f32 * NEXT_PREVIEW_SLOT_H;
        tf.translation = preview_block_pos(panel_x, slot_y, kind, block.index);
        sprite.color = active_color(piece.player, kind);
    }

    for (block, mut tf, mut sprite) in &mut hold_blocks {
        let panel_x = if block.player == PlayerId::P1 { P1_PANEL_X } else { P2_PANEL_X };
        let Some((piece, _)) = players.iter().find(|(p, _)| p.player == block.player) else {
            tf.translation.y = -1000.0;
            continue;
        };
        match piece.hold {
            Some(kind) => {
                tf.translation = preview_block_pos(panel_x, HOLD_PREVIEW_Y, kind, block.index);
                let mut col = active_color(piece.player, kind).to_srgba();
                // Dim the hold piece if hold is locked for this turn
                if piece.hold_used {
                    col.red *= 0.5;
                    col.green *= 0.5;
                    col.blue *= 0.5;
                }
                sprite.color = Color::srgba(col.red, col.green, col.blue, col.alpha);
            }
            None => {
                tf.translation.y = -1000.0;
                sprite.color = Color::NONE;
            }
        }
    }
}

/// Populate PieceLockFlash when a piece locks.
pub fn on_piece_locked(
    mut ev: EventReader<PieceLocked>,
    mut lock_flash: ResMut<PieceLockFlash>,
) {
    for event in ev.read() {
        lock_flash.timer = LOCK_FLASH_DURATION;
        lock_flash.cells = event
            .cells
            .iter()
            .filter(|&&(_, row)| row >= 0 && row < VISIBLE_ROWS as i32)
            .map(|&(col, row)| (col as usize, row as usize))
            .collect();
    }
}

/// Tick the lock flash timer and clear cells when it expires.
pub fn tick_lock_flash(time: Res<Time>, mut lock_flash: ResMut<PieceLockFlash>) {
    if lock_flash.timer > 0.0 {
        lock_flash.timer = (lock_flash.timer - time.delta_secs()).max(0.0);
        if lock_flash.timer == 0.0 {
            lock_flash.cells.clear();
        }
    }
}

// ── Popup helpers ──────────────────────────────────────────────────────────────

fn spawn_popup(commands: &mut Commands, text: &str, pos: Vec3, lifetime: f32) {
    commands.spawn((
        Text2d::new(text.to_string()),
        TextFont::from_font_size(28.0),
        TextColor(Color::WHITE),
        Transform::from_translation(pos),
        PopupText { lifetime, total: lifetime },
    ));
}

/// Spawn action popups (TETRIS!, T-SPIN!, COMBO ×N, DOUBLE, TRIPLE) on line clears.
pub fn spawn_popups(
    mut commands: Commands,
    mut ev: EventReader<LinesCleared>,
    score: Res<ScoreBoard>,
) {
    for event in ev.read() {
        if event.count == 0 {
            continue;
        }
        let x = match event.player {
            PlayerId::P1 => -200.0_f32,
            PlayerId::P2 => 200.0_f32,
        };
        let label = match (event.count, event.t_spin) {
            (4, TSpinType::None) => Some("TETRIS!"),
            (_, TSpinType::Full)  => Some("T-SPIN!"),
            (_, TSpinType::Mini)  => Some("T-SPIN MINI"),
            (3, _) => Some("TRIPLE"),
            (2, _) => Some("DOUBLE"),
            _ => None,
        };
        if let Some(text) = label {
            spawn_popup(&mut commands, text, Vec3::new(x, 40.0, 10.0), 1.5);
        }
        if score.combo > 0 {
            let combo_text = format!("COMBO x{}", score.combo);
            spawn_popup(&mut commands, &combo_text, Vec3::new(x, 10.0, 10.0), 1.5);
        }
    }
}

/// Spawn "LEVEL X!" popup when level increases.
pub fn on_level_up(mut commands: Commands, mut ev: EventReader<LevelUpEvent>) {
    for event in ev.read() {
        let text = format!("LEVEL {}!", event.new_level);
        spawn_popup(&mut commands, &text, Vec3::new(0.0, 80.0, 10.0), 2.0);
    }
}

/// Move popups upward and fade them out; despawn when expired.
pub fn tick_popups(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut TextColor, &mut PopupText)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut color, mut popup) in &mut query {
        popup.lifetime -= dt;
        if popup.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        tf.translation.y += 40.0 * dt;
        // Fade out in the last 50 % of the lifetime
        let fade_start = popup.total * 0.5;
        let alpha = if popup.lifetime < fade_start {
            popup.lifetime / fade_start
        } else {
            1.0
        };
        color.0 = Color::srgba(1.0, 1.0, 1.0, alpha);
    }
}
