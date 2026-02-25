use bevy::prelude::*;

use crate::input::PieceAction;
use crate::player::PlayerId;

/// Emitted when the player chooses "Quit to Menu" from the pause screen.
/// Consumed by the `cleanup_on_quit` system on `OnExit(Paused)`.
#[derive(Event)]
pub struct QuitToMenu;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    ModeSelect,
    Settings,
    Playing,
    Paused,
    GameOver,
    SprintComplete,
}

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum SelectedMode {
    #[default]
    Endless,
    Sprint,
    Ultra,
}

/// Tracks which binding is waiting to be captured (player + action pair).
#[derive(Resource, Default)]
pub struct RebindTarget {
    pub pending: Option<(PlayerId, PieceAction)>,
}
