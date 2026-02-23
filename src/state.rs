use bevy::prelude::*;

/// Set to true when the player chooses "Quit to Menu" from the pause screen.
/// Consumed by the `cleanup_on_quit` system on `OnExit(Paused)`.
#[derive(Resource, Default)]
pub struct QuitToMenu(pub bool);

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    Paused,
    GameOver,
}
