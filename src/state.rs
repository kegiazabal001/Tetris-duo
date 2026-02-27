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
    Chaos,
}

pub const CHAOS_PIECE_THRESHOLD: u32 = 10;
pub const CHAOS_SWAP_PIECES: u32     = 3;
pub const CHAOS_BLACKOUT_SECS: f32   = 8.0;

/// All possible chaos events. To add a new one: add a variant here,
/// new fields in ChaosState, and a new match arm in trigger/tick logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChaosEvent {
    SwapBlackout,
}

#[derive(Resource, Default)]
pub struct ChaosState {
    /// Pieces placed since last event ended (only counts when no event active).
    pub pieces_since_last_event: u32,
    /// Currently active event, if any.
    pub active_event: Option<ChaosEvent>,
    // SwapBlackout fields
    pub swap_p1_remaining: u32,
    pub swap_p2_remaining: u32,
    pub swap_active:       bool,
    pub blackout_timer:    f32,
    pub blackout_active:   bool,
}

impl ChaosState {
    /// Trigger the next random chaos event.
    /// When more events are added, introduce rand selection here.
    pub fn trigger_next_event(&mut self) {
        self.active_event      = Some(ChaosEvent::SwapBlackout);
        self.swap_p1_remaining = CHAOS_SWAP_PIECES;
        self.swap_p2_remaining = CHAOS_SWAP_PIECES;
        self.swap_active       = true;
        self.blackout_timer    = CHAOS_BLACKOUT_SECS;
        self.blackout_active   = true;
    }
}

/// Tracks which binding is waiting to be captured (player + action pair).
#[derive(Resource, Default)]
pub struct RebindTarget {
    pub pending: Option<(PlayerId, PieceAction)>,
}
