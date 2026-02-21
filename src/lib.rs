pub mod board;
pub mod collision;
pub mod input;
pub mod piece;
pub mod player;
pub mod render;
pub mod scoring;
pub mod state;
pub mod ui;

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::input::PieceAction;
use crate::state::GameState;

pub struct TetrisDuoPlugin;

impl Plugin for TetrisDuoPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins(InputManagerPlugin::<PieceAction>::default())
            .init_resource::<board::Board>()
            .init_resource::<scoring::ScoreBoard>()
            .init_resource::<render::LineClearFlash>()
            .init_resource::<render::PieceLockFlash>()
            .add_event::<player::PieceLocked>()
            .add_event::<player::LinesCleared>()
            .add_event::<player::PieceRotated>()
            .add_event::<player::GameOverEvent>()
            .add_event::<scoring::LevelUpEvent>()
            // Menu
            .add_systems(OnEnter(GameState::Menu), ui::setup_menu)
            .add_systems(OnExit(GameState::Menu), ui::despawn_menu)
            .add_systems(Update, ui::menu_input.run_if(in_state(GameState::Menu)))
            // Playing: enter/exit
            .add_systems(
                OnEnter(GameState::Playing),
                (player::spawn_players, render::setup_board_visuals, ui::setup_hud),
            )
            .add_systems(
                OnExit(GameState::Playing),
                (player::despawn_players, render::despawn_board_visuals, ui::despawn_hud),
            )
            // Playing: game loop
            .add_systems(
                Update,
                (
                    player::handle_input,
                    player::apply_gravity,
                    player::check_lock,
                    render::on_piece_locked,
                    player::lock_piece,
                    scoring::update_score,
                    render::on_lines_cleared,
                    player::check_game_over,
                    render::sync_board_cells,
                    render::sync_active_pieces,
                    render::sync_ghost_pieces,
                    render::sync_preview_pieces,
                    ui::update_hud,
                    render::tick_flash_timer,
                    render::tick_lock_flash,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            // Pause
            .add_systems(OnEnter(GameState::Paused), ui::setup_pause)
            .add_systems(OnExit(GameState::Paused), ui::despawn_pause)
            .add_systems(Update, ui::pause_input)
            // Game Over
            .add_systems(OnEnter(GameState::GameOver), ui::setup_game_over)
            .add_systems(OnExit(GameState::GameOver), ui::despawn_game_over)
            .add_systems(
                Update,
                ui::game_over_input.run_if(in_state(GameState::GameOver)),
            );
    }
}
