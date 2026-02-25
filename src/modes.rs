use bevy::prelude::*;

use crate::config::AppConfig;
use crate::scoring::ScoreBoard;
use crate::state::{GameState, SelectedMode};

pub const SPRINT_GOAL: u32 = 20;
pub const ULTRA_DURATION: f32 = 120.0;

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
    if score.score > config.high_scores.ultra_best {
        config.high_scores.ultra_best = score.score;
        config.save();
    }
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
