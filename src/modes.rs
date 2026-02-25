use bevy::prelude::*;

use crate::scoring::ScoreBoard;
use crate::state::{GameState, SelectedMode};

pub const SPRINT_GOAL: u32 = 40;
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

/// Sprint victory condition: 40 lines cleared → SprintComplete.
pub fn check_sprint_complete(
    mode: Res<SelectedMode>,
    score: Res<ScoreBoard>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if *mode == SelectedMode::Sprint && score.lines_cleared >= SPRINT_GOAL {
        next_state.set(GameState::SprintComplete);
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
    fn sprint_goal_constant_is_40() {
        assert_eq!(SPRINT_GOAL, 40);
    }

    #[test]
    fn ultra_remaining_clamps_at_zero() {
        let elapsed = 130.0_f32;
        let remaining = (ULTRA_DURATION - elapsed).max(0.0);
        assert_eq!(remaining, 0.0);
    }
}
