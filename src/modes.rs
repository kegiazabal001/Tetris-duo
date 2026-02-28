use bevy::prelude::*;

use crate::board::Board;
use crate::config::AppConfig;
use crate::player::{PieceLocked, PlayerId};
use crate::scoring::ScoreBoard;
use crate::state::{ChaosEvent, ChaosState, FlipPhase, GameState, SelectedMode, CHAOS_PIECE_THRESHOLD};

pub use crate::constants::{FLIP_DURATION, FLIP_MIGRATION_INTERVAL, SPRINT_GOAL, ULTRA_DURATION};

#[derive(Resource, Default)]
pub struct ModeTimer {
    pub elapsed: f32,
}

/// Resets the timer when a game session starts.
pub fn start_mode_timer(mut timer: ResMut<ModeTimer>) {
    timer.elapsed = 0.0;
}

/// Advances the timer every frame while in Playing state.
pub fn tick_mode_timer(mut timer: ResMut<ModeTimer>, time: Res<Time>) {
    timer.elapsed += time.delta_secs();
}

/// Sprint victory condition: 20 lines cleared → SprintComplete.
pub fn check_sprint_complete(
    mode: Res<SelectedMode>,
    score: Res<ScoreBoard>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if *mode == SelectedMode::Sprint && score.lines_cleared >= SPRINT_GOAL {
        next_state.set(GameState::SprintComplete);
    }
}

/// Saves Ultra best score when GameOver is entered (any cause) in Ultra mode.
pub fn save_ultra_score(
    mode: Res<SelectedMode>,
    score: Res<ScoreBoard>,
    mut config: ResMut<AppConfig>,
) {
    if *mode != SelectedMode::Ultra {
        return;
    }
    config.try_update_ultra(score.score);
}

/// Ultra timeout condition: 120 s elapsed → GameOver.
pub fn check_ultra_timeout(
    mode: Res<SelectedMode>,
    timer: Res<ModeTimer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if *mode == SelectedMode::Ultra && timer.elapsed >= ULTRA_DURATION {
        next_state.set(GameState::GameOver);
    }
}

/// Resets ChaosState at the start of each game session.
pub fn reset_chaos_state(mut cs: ResMut<ChaosState>) {
    *cs = ChaosState::default();
}

/// Resolves chaos event completion: swap ends when both players have placed all swapped pieces.
pub fn tick_chaos(
    mode: Res<SelectedMode>,
    mut cs: ResMut<ChaosState>,
) {
    if *mode != SelectedMode::Chaos { return; }
    if cs.active_event.is_none() { return; }

    if cs.swap_active && cs.swap_p1_remaining == 0 && cs.swap_p2_remaining == 0 {
        cs.swap_active             = false;
        cs.active_event            = None;
        cs.pieces_since_last_event = 0;
    }
}

/// Reacts to piece-lock events to drive the chaos event counter and swap tracking.
pub fn on_piece_locked_chaos(
    mode:    Res<SelectedMode>,
    mut cs:  ResMut<ChaosState>,
    mut ev:  EventReader<PieceLocked>,
) {
    if *mode != SelectedMode::Chaos { return; }

    for event in ev.read() {
        match cs.active_event {
            Some(ChaosEvent::SwapBlackout) => {
                match event.player {
                    PlayerId::P1 => { if cs.swap_p1_remaining > 0 { cs.swap_p1_remaining -= 1; } }
                    PlayerId::P2 => { if cs.swap_p2_remaining > 0 { cs.swap_p2_remaining -= 1; } }
                }
            }
            Some(ChaosEvent::Flip) => {
                // Waiting phases: track when each player places their current piece.
                // The actual phase advance happens in tick_flip_migration.
                match cs.flip_phase {
                    FlipPhase::Waiting | FlipPhase::WaitingEnd => {
                        match event.player {
                            PlayerId::P1 => cs.flip_wait_p1_done = true,
                            PlayerId::P2 => cs.flip_wait_p2_done = true,
                        }
                    }
                    _ => {}
                }
            }
            None => {
                cs.pieces_since_last_event += 1;
                if cs.pieces_since_last_event >= CHAOS_PIECE_THRESHOLD {
                    cs.trigger_next_event();
                }
            }
        }
    }
}

/// Drives the FLIP! event state machine every frame.
pub fn tick_flip_migration(
    mode:     Res<SelectedMode>,
    time:     Res<Time>,
    mut cs:   ResMut<ChaosState>,
    mut board: ResMut<Board>,
    mut score: ResMut<ScoreBoard>,
) {
    if *mode != SelectedMode::Chaos { return; }
    if cs.active_event != Some(ChaosEvent::Flip) { return; }

    let dt = time.delta_secs();

    match cs.flip_phase {
        FlipPhase::Waiting => {
            if cs.flip_wait_p1_done && cs.flip_wait_p2_done {
                cs.flip_phase = FlipPhase::MigratingUp;
                cs.flip_timer = 0.0;
            }
        }
        FlipPhase::MigratingUp => {
            cs.flip_timer += dt;
            if cs.flip_timer >= FLIP_MIGRATION_INTERVAL {
                cs.flip_timer -= FLIP_MIGRATION_INTERVAL;
                let moved = board.apply_gravity_step(1);
                // Clear any full rows formed by compaction against the ceiling.
                let full = board.detect_full_rows();
                if !full.is_empty() {
                    let n = full.len() as u32;
                    board.remove_rows(&full, 1);
                    score.score += n * 100; // bonus for lines cleared during migration
                }
                if !moved {
                    // All cells have reached the ceiling — begin Active phase.
                    cs.flip_phase = FlipPhase::Active;
                    cs.gravity_dir = -1;
                    cs.flip_timer = FLIP_DURATION;
                    // Reset waiting flags for WaitingEnd.
                    cs.flip_wait_p1_done = false;
                    cs.flip_wait_p2_done = false;
                }
            }
        }
        FlipPhase::Active => {
            cs.flip_timer -= dt;
            if cs.flip_timer <= 0.0 {
                cs.flip_phase = FlipPhase::WaitingEnd;
            }
        }
        FlipPhase::WaitingEnd => {
            if cs.flip_wait_p1_done && cs.flip_wait_p2_done {
                cs.flip_phase = FlipPhase::MigratingDown;
                cs.flip_timer = 0.0;
                cs.gravity_dir = 1; // restore normal gravity for migration
            }
        }
        FlipPhase::MigratingDown => {
            cs.flip_timer += dt;
            if cs.flip_timer >= FLIP_MIGRATION_INTERVAL {
                cs.flip_timer -= FLIP_MIGRATION_INTERVAL;
                let moved = board.apply_gravity_step(-1);
                // Clear any full rows formed by compaction toward the floor.
                let full = board.detect_full_rows();
                if !full.is_empty() {
                    let n = full.len() as u32;
                    board.remove_rows(&full, -1);
                    score.score += n * 100;
                }
                if !moved {
                    // All cells have settled — event complete.
                    cs.flip_phase = FlipPhase::Inactive;
                    cs.active_event = None;
                    cs.pieces_since_last_event = 0;
                    score.score += 500; // survived the event
                }
            }
        }
        FlipPhase::Inactive => {}
    }
}

/// Saves Chaos best score when GameOver is entered in Chaos mode.
pub fn save_chaos_score(
    mode:       Res<SelectedMode>,
    score:      Res<ScoreBoard>,
    mut config: ResMut<AppConfig>,
) {
    if *mode != SelectedMode::Chaos { return; }
    config.try_update_chaos(score.score);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprint_goal_constant_is_20() {
        assert_eq!(SPRINT_GOAL, 20);
    }

    #[test]
    fn ultra_remaining_clamps_at_zero() {
        let elapsed = 130.0_f32;
        let remaining = (ULTRA_DURATION - elapsed).max(0.0);
        assert_eq!(remaining, 0.0);
    }
}
