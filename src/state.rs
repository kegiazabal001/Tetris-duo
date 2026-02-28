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

/// All possible chaos events. To add a new one: add a variant here,
/// new fields in ChaosState, and a new match arm in trigger/tick logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChaosEvent {
    SwapBlackout,
    Flip,
}

/// Phase of the FLIP! event state machine.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlipPhase {
    /// No FLIP event active.
    #[default]
    Inactive,
    /// Waiting for both players to place their current piece before flipping.
    Waiting,
    /// Gravity inverted; active gameplay for FLIP_DURATION seconds.
    Active,
    /// Waiting for both players to place their current piece before flipping back.
    WaitingEnd,
}

#[derive(Resource)]
pub struct ChaosState {
    /// Pieces placed since last event ended (only counts when no event active).
    pub pieces_since_last_event: u32,
    /// Currently active event, if any.
    pub active_event: Option<ChaosEvent>,
    // SwapBlackout fields
    pub swap_p1_remaining: u32,
    pub swap_p2_remaining: u32,
    pub swap_active:       bool,
    // FLIP fields
    pub flip_phase:        FlipPhase,
    /// Countdown during Active; accumulator during migrations.
    pub flip_timer:        f32,
    pub flip_wait_p1_done: bool,
    pub flip_wait_p2_done: bool,
    /// Gravity direction: +1 = normal (down), -1 = inverted (up).
    pub gravity_dir:       i8,
}

impl Default for ChaosState {
    fn default() -> Self {
        Self {
            pieces_since_last_event: 0,
            active_event: None,
            swap_p1_remaining: 0,
            swap_p2_remaining: 0,
            swap_active: false,
            flip_phase: FlipPhase::Inactive,
            flip_timer: 0.0,
            flip_wait_p1_done: false,
            flip_wait_p2_done: false,
            gravity_dir: 1,
        }
    }
}

impl ChaosState {
    /// Trigger the next random chaos event.
    pub fn trigger_next_event(&mut self) {
        if rand::random::<bool>() {
            self.active_event      = Some(ChaosEvent::SwapBlackout);
            self.swap_p1_remaining = CHAOS_SWAP_PIECES;
            self.swap_p2_remaining = CHAOS_SWAP_PIECES;
            self.swap_active       = true;
        } else {
            self.active_event      = Some(ChaosEvent::Flip);
            self.flip_phase        = FlipPhase::Waiting;
            self.flip_wait_p1_done = false;
            self.flip_wait_p2_done = false;
            self.flip_timer        = crate::constants::FLIP_DURATION;
        }
    }

    /// True if any chaos event is blocking normal piece spawning.
    pub fn blocks_spawn(&self) -> bool {
        matches!(self.flip_phase, FlipPhase::Waiting | FlipPhase::WaitingEnd)
    }
}

/// Tracks which binding is waiting to be captured (player + action pair).
#[derive(Resource, Default)]
pub struct RebindTarget {
    pub pending: Option<(PlayerId, PieceAction)>,
}
