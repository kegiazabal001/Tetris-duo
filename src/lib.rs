pub mod audio;
pub mod board;
pub mod collision;
pub mod config;
pub mod input;
pub mod modes;
pub mod piece;
pub mod player;
pub mod render;
pub mod scoring;
pub mod state;
pub mod ui;

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::config::AppConfig;
use crate::input::PieceAction;
use crate::player::InputGrace;
use crate::state::{GameState, QuitToMenu, SelectedMode};

pub struct TetrisDuoPlugin;

impl Plugin for TetrisDuoPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins(InputManagerPlugin::<PieceAction>::default())
            .init_resource::<AppConfig>()
            .init_resource::<board::Board>()
            .init_resource::<scoring::ScoreBoard>()
            .init_resource::<InputGrace>()
            .init_resource::<render::LineClearFlash>()
            .init_resource::<render::PieceLockFlash>()
            .init_resource::<QuitToMenu>()
            .init_resource::<SelectedMode>()
            .init_resource::<modes::ModeTimer>()
            .add_event::<player::PieceLocked>()
            .add_event::<player::LinesCleared>()
            .add_event::<player::PieceRotated>()
            .add_event::<player::GameOverEvent>()
            .add_event::<scoring::LevelUpEvent>()
            // Startup
            .add_systems(PreStartup, (audio::load_audio, config::load_config))
            .add_systems(Startup, audio::start_bg_music)
            // Menu
            .add_systems(OnEnter(GameState::Menu), ui::setup_menu)
            .add_systems(OnExit(GameState::Menu), ui::despawn_menu)
            .add_systems(Update, ui::menu_input.run_if(in_state(GameState::Menu)))
            // ModeSelect
            .add_systems(OnEnter(GameState::ModeSelect), ui::setup_mode_select)
            .add_systems(OnExit(GameState::ModeSelect), ui::despawn_mode_select)
            .add_systems(
                Update,
                ui::mode_select_input.run_if(in_state(GameState::ModeSelect)),
            )
            // Playing: enter/exit
            .add_systems(
                OnEnter(GameState::Playing),
                (
                    modes::start_mode_timer,
                    player::spawn_players,
                    render::setup_board_visuals,
                    ui::setup_hud,
                    |mut grace: ResMut<InputGrace>| grace.0 = true,
                ),
            )
            .add_systems(
                OnExit(GameState::Playing),
                (player::despawn_players, render::despawn_board_visuals, ui::despawn_hud),
            )
            // Playing: game loop
            .add_systems(
                Update,
                (
                    (
                        player::handle_input,
                        player::apply_gravity,
                        player::check_lock,
                        render::on_piece_locked,
                        player::lock_piece,
                        scoring::update_score,
                        render::on_lines_cleared,
                        render::spawn_popups,
                        render::on_level_up,
                        scoring::save_high_score,
                        audio::play_piece_sounds,
                        audio::play_rotate_sound,
                    ),
                    (
                        audio::play_level_up_sound,
                        audio::play_game_over_sound,
                        player::check_game_over,
                        modes::tick_mode_timer,
                        modes::check_sprint_complete,
                        modes::check_ultra_timeout,
                        render::sync_board_cells,
                        render::sync_active_pieces,
                        render::sync_ghost_pieces,
                        render::sync_preview_pieces,
                        ui::update_hud,
                        render::tick_flash_timer,
                        render::tick_lock_flash,
                        render::tick_popups,
                    ),
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            // Pause
            .add_systems(OnEnter(GameState::Paused), (ui::setup_pause, audio::pause_bg_music))
            .add_systems(
                OnExit(GameState::Paused),
                (ui::despawn_pause, cleanup_on_quit, audio::resume_bg_music),
            )
            .add_systems(Update, ui::pause_input)
            // Game Over
            .add_systems(OnEnter(GameState::GameOver), (ui::setup_game_over, audio::stop_bg_music))
            .add_systems(OnExit(GameState::GameOver), (ui::despawn_game_over, audio::start_bg_music))
            .add_systems(
                Update,
                ui::game_over_input.run_if(in_state(GameState::GameOver)),
            )
            // Sprint Complete
            .add_systems(OnEnter(GameState::SprintComplete), ui::setup_sprint_complete)
            .add_systems(OnExit(GameState::SprintComplete), ui::despawn_sprint_complete)
            .add_systems(
                Update,
                ui::sprint_complete_input.run_if(in_state(GameState::SprintComplete)),
            );
    }
}

/// Cleans up all in-game entities when the player quits to menu from the pause screen.
/// Runs on `OnExit(Paused)`; does nothing when simply resuming.
fn cleanup_on_quit(
    mut quit: ResMut<QuitToMenu>,
    mut commands: Commands,
    mut board: ResMut<board::Board>,
    mut score: ResMut<scoring::ScoreBoard>,
    mut mode_timer: ResMut<modes::ModeTimer>,
    mut selected_mode: ResMut<SelectedMode>,
    players: Query<Entity, With<player::ActivePiece>>,
    board_sprites: Query<
        Entity,
        Or<(
            With<render::BoardCellSprite>,
            With<render::ActiveBlockSprite>,
            With<render::GhostBlockSprite>,
            With<render::BoardBackdrop>,
            With<render::NextPieceBlock>,
            With<render::HoldPieceBlock>,
            With<render::PopupText>,
        )>,
    >,
    hud: Query<Entity, With<ui::HudRoot>>,
) {
    if !quit.0 {
        return;
    }
    quit.0 = false;
    for e in players.iter().chain(board_sprites.iter()).chain(hud.iter()) {
        commands.entity(e).despawn();
    }
    *board = board::Board::default();
    score.reset_preserving_high_score();
    mode_timer.elapsed = 0.0;
    *selected_mode = SelectedMode::default();
}
