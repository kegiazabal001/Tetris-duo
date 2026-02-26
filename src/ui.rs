use bevy::prelude::*;

use crate::board::Board;
use crate::config::AppConfig;
use crate::modes::{ModeTimer, SPRINT_GOAL, ULTRA_DURATION};
use crate::scoring::ScoreBoard;
use crate::state::{GameState, QuitToMenu, SelectedMode};

fn reset_game(score: &mut ScoreBoard, board: &mut Board, start_level: u32) {
    score.reset_preserving_high_score();
    score.level = start_level.max(1);
    *board = Board::default();
}

// ── HUD ──────────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct LevelText;

#[derive(Component)]
pub struct ComboText;

#[derive(Component)]
pub struct HighScoreText;

#[derive(Component)]
pub struct ModeTimerText;

#[derive(Component)]
pub struct LinesRemainingText;

pub fn setup_hud(mut commands: Commands, mode: Res<SelectedMode>) {
    commands
        .spawn((
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Px(12.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(6.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                ScoreText,
                Text::new("Score: 0"),
                TextColor(Color::WHITE),
                TextFont::from_font_size(22.0),
            ));
            parent.spawn((
                LevelText,
                Text::new("Level: 1"),
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                TextFont::from_font_size(18.0),
            ));
            parent.spawn((
                ComboText,
                Text::new(""),
                TextColor(Color::srgb(1.0, 0.85, 0.2)),
                TextFont::from_font_size(16.0),
            ));
            parent.spawn((
                HighScoreText,
                Text::new("Best: 0"),
                TextColor(Color::srgb(0.6, 0.9, 1.0)),
                TextFont::from_font_size(16.0),
            ));
            match *mode {
                SelectedMode::Sprint => {
                    parent.spawn((
                        LinesRemainingText,
                        Text::new(format!("Lines: {SPRINT_GOAL}")),
                        TextColor(Color::srgb(0.4, 1.0, 0.6)),
                        TextFont::from_font_size(20.0),
                    ));
                    parent.spawn((
                        ModeTimerText,
                        Text::new("00:00.0"),
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        TextFont::from_font_size(20.0),
                    ));
                }
                SelectedMode::Ultra => {
                    parent.spawn((
                        ModeTimerText,
                        Text::new("02:00"),
                        TextColor(Color::WHITE),
                        TextFont::from_font_size(22.0),
                    ));
                }
                SelectedMode::Endless => {}
            }
        });
}

pub fn despawn_hud(mut commands: Commands, query: Query<Entity, With<HudRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[allow(clippy::type_complexity)]
pub fn update_hud(
    score: Res<ScoreBoard>,
    mode: Res<SelectedMode>,
    timer: Res<ModeTimer>,
    mut score_q: Query<
        &mut Text,
        (
            With<ScoreText>,
            Without<LevelText>,
            Without<ComboText>,
            Without<HighScoreText>,
            Without<ModeTimerText>,
            Without<LinesRemainingText>,
        ),
    >,
    mut level_q: Query<
        &mut Text,
        (
            With<LevelText>,
            Without<ScoreText>,
            Without<ComboText>,
            Without<HighScoreText>,
            Without<ModeTimerText>,
            Without<LinesRemainingText>,
        ),
    >,
    mut combo_q: Query<
        &mut Text,
        (
            With<ComboText>,
            Without<ScoreText>,
            Without<LevelText>,
            Without<HighScoreText>,
            Without<ModeTimerText>,
            Without<LinesRemainingText>,
        ),
    >,
    mut hs_q: Query<
        &mut Text,
        (
            With<HighScoreText>,
            Without<ScoreText>,
            Without<LevelText>,
            Without<ComboText>,
            Without<ModeTimerText>,
            Without<LinesRemainingText>,
        ),
    >,
    mut lines_q: Query<
        &mut Text,
        (
            With<LinesRemainingText>,
            Without<ScoreText>,
            Without<LevelText>,
            Without<ComboText>,
            Without<HighScoreText>,
            Without<ModeTimerText>,
        ),
    >,
    mut mode_timer_q: Query<
        (&mut Text, &mut TextColor),
        (
            With<ModeTimerText>,
            Without<ScoreText>,
            Without<LevelText>,
            Without<ComboText>,
            Without<HighScoreText>,
            Without<LinesRemainingText>,
        ),
    >,
) {
    for mut text in &mut score_q {
        **text = format!("Score: {}", fmt_score(score.score));
    }
    for mut text in &mut level_q {
        **text = format!("Level: {}", score.level);
    }
    for mut text in &mut combo_q {
        **text = if score.combo > 0 {
            format!("Combo x{}", score.combo + 1)
        } else {
            String::new()
        };
    }
    for mut text in &mut hs_q {
        **text = format!("Best: {}", fmt_score(score.high_score));
    }

    match *mode {
        SelectedMode::Sprint => {
            let remaining = SPRINT_GOAL.saturating_sub(score.lines_cleared);
            for mut text in &mut lines_q {
                **text = format!("Lines: {remaining}");
            }
            for (mut text, _) in &mut mode_timer_q {
                **text = fmt_time_sprint(timer.elapsed);
            }
        }
        SelectedMode::Ultra => {
            let remaining = (ULTRA_DURATION - timer.elapsed).max(0.0);
            let total_secs = remaining as u32;
            // Progressive color: white → red as remaining goes from 30s to 0s.
            let color = if remaining >= 30.0 {
                Color::WHITE
            } else {
                // t = 1.0 at 30s remaining, 0.0 at 0s → lerp green/blue channel down
                let t = remaining / 30.0; // 0..=1
                Color::srgb(1.0, t * 1.0, t * 1.0)
            };
            for (mut text, mut text_color) in &mut mode_timer_q {
                **text = format!("{:02}:{:02}", total_secs / 60, total_secs % 60);
                *text_color = TextColor(color);
            }
        }
        SelectedMode::Endless => {}
    }
}

// ── MENU ─────────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct MenuRoot;

pub fn setup_menu(mut commands: Commands) {
    commands
        .spawn((
            MenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(14.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("TETRIS DUO"),
                TextColor(Color::WHITE),
                TextFont::from_font_size(52.0),
            ));
            parent.spawn((
                Text::new("Cooperative - clear lines together!"),
                TextColor(Color::srgb(0.6, 0.9, 0.6)),
                TextFont::from_font_size(18.0),
            ));
            // Spacer
            parent.spawn((
                Text::new(" "),
                TextFont::from_font_size(8.0),
            ));
            parent.spawn((
                Text::new("SPACE  - select mode"),
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                TextFont::from_font_size(22.0),
            ));
            parent.spawn((
                Text::new("M      - mute / unmute"),
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                TextFont::from_font_size(18.0),
            ));
            // Spacer
            parent.spawn((
                Text::new(" "),
                TextFont::from_font_size(8.0),
            ));
            parent.spawn((
                Text::new("P1: A/D move  W/Q rotate  S soft-drop  Space hard-drop  LShift hold"),
                TextColor(Color::srgb(0.4, 0.65, 1.0)),
                TextFont::from_font_size(14.0),
            ));
            parent.spawn((
                Text::new("P2: Left/Right move  Up/RCtrl rotate  Down soft-drop  Enter hard-drop  RShift hold"),
                TextColor(Color::srgb(1.0, 0.75, 0.55)),
                TextFont::from_font_size(14.0),
            ));
        });
}

pub fn despawn_menu(mut commands: Commands, query: Query<Entity, With<MenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Menu → ModeSelect (SPACE).
pub fn menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(GameState::ModeSelect);
    }
}

// ── MODE SELECT ───────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct ModeSelectRoot;

#[derive(Component)]
pub struct StartLevelText;

pub fn setup_mode_select(mut commands: Commands, config: Res<AppConfig>) {
    let endless_best = format!("Best: {} pts", fmt_score(config.high_scores.endless));
    let sprint_best = match config.high_scores.sprint_best {
        Some(secs) => format!("Best: {}", fmt_time_sprint(secs)),
        None => "Best: --".to_string(),
    };
    let ultra_best = format!("Best: {} pts", fmt_score(config.high_scores.ultra_best));

    commands
        .spawn((
            ModeSelectRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.88)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("SELECT MODE"),
                TextColor(Color::WHITE),
                TextFont::from_font_size(40.0),
            ));

            // Endless
            parent.spawn((
                Text::new(format!("1  CLASIC  -  {endless_best}")),
                TextColor(Color::srgb(0.6, 0.9, 1.0)),
                TextFont::from_font_size(24.0),
            ));
            // Sprint
            parent.spawn((
                Text::new(format!("2  SPRINT  ({} lines)  -  {sprint_best}", SPRINT_GOAL)),
                TextColor(Color::srgb(0.4, 1.0, 0.6)),
                TextFont::from_font_size(24.0),
            ));
            // Ultra
            parent.spawn((
                Text::new(format!("3  ULTRA  (2 min)  -  {ultra_best}")),
                TextColor(Color::srgb(1.0, 0.7, 0.3)),
                TextFont::from_font_size(24.0),
            ));

            parent.spawn((
                StartLevelText,
                Text::new(format!("Nivel inicio: {:2}  ( <- -> )", config.start_level)),
                TextColor(Color::srgb(0.9, 0.9, 0.5)),
                TextFont::from_font_size(20.0),
            ));

            parent.spawn((
                Text::new("S - Settings"),
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                TextFont::from_font_size(18.0),
            ));
        });
}

pub fn despawn_mode_select(mut commands: Commands, query: Query<Entity, With<ModeSelectRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn mode_select_level_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<AppConfig>,
    mut text_q: Query<&mut Text, With<StartLevelText>>,
) {
    let changed = if keyboard.just_pressed(KeyCode::ArrowLeft) {
        if config.start_level > 1 { config.start_level -= 1; true } else { false }
    } else if keyboard.just_pressed(KeyCode::ArrowRight) {
        if config.start_level < 15 { config.start_level += 1; true } else { false }
    } else {
        false
    };

    if changed {
        config.save();
        if let Ok(mut text) = text_q.single_mut() {
            *text = Text::new(format!("Nivel inicio: {:2}  ( <- -> )", config.start_level));
        }
    }
}

pub fn mode_select_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut selected_mode: ResMut<SelectedMode>,
    mut score: ResMut<ScoreBoard>,
    mut board: ResMut<Board>,
    config: Res<AppConfig>,
) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        *selected_mode = SelectedMode::Endless;
        reset_game(&mut score, &mut board, config.start_level);
        next_state.set(GameState::Playing);
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        *selected_mode = SelectedMode::Sprint;
        reset_game(&mut score, &mut board, config.start_level);
        next_state.set(GameState::Playing);
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        *selected_mode = SelectedMode::Ultra;
        reset_game(&mut score, &mut board, config.start_level);
        next_state.set(GameState::Playing);
    } else if keyboard.just_pressed(KeyCode::KeyS) {
        next_state.set(GameState::Settings);
    }
}

// ── PAUSE ─────────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct PauseRoot;

pub fn setup_pause(mut commands: Commands) {
    commands
        .spawn((
            PauseRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED"),
                TextColor(Color::srgb(1.0, 0.9, 0.2)),
                TextFont::from_font_size(56.0),
            ));
            parent.spawn((
                Text::new("ESC - resume"),
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                TextFont::from_font_size(22.0),
            ));
            parent.spawn((
                Text::new("M   - mute / unmute"),
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                TextFont::from_font_size(20.0),
            ));
            parent.spawn((
                Text::new("Q   - quit to menu"),
                TextColor(Color::srgb(0.9, 0.4, 0.4)),
                TextFont::from_font_size(22.0),
            ));
        });
}

pub fn despawn_pause(mut commands: Commands, query: Query<Entity, With<PauseRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut quit_to_menu: EventWriter<QuitToMenu>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match state.get() {
            GameState::Playing => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Playing),
            _ => {}
        }
    }
    if keyboard.just_pressed(KeyCode::KeyQ) {
        if *state.get() == GameState::Paused {
            quit_to_menu.write(QuitToMenu);
            next_state.set(GameState::Menu);
        }
    }
}

// ── GAME OVER ─────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct GameOverRoot;

pub fn setup_game_over(
    mut commands: Commands,
    score: Res<ScoreBoard>,
    mode: Res<SelectedMode>,
    timer: Res<ModeTimer>,
    config: Res<AppConfig>,
) {
    let is_timeout = *mode == SelectedMode::Ultra && timer.elapsed >= ULTRA_DURATION;

    let (title, title_color) = if is_timeout {
        ("TIME'S UP!", Color::srgb(1.0, 0.6, 0.1))
    } else {
        ("GAME OVER", Color::srgb(1.0, 0.3, 0.3))
    };

    let best_line = match *mode {
        SelectedMode::Endless => {
            format!("Best: {} pts", fmt_score(config.high_scores.endless.max(score.score)))
        }
        SelectedMode::Sprint => match config.high_scores.sprint_best {
            Some(secs) => format!("Best: {}", fmt_time_sprint(secs)),
            None => "Best: --".to_string(),
        },
        SelectedMode::Ultra => {
            format!("Best: {} pts", fmt_score(config.high_scores.ultra_best.max(score.score)))
        }
    };

    commands
        .spawn((
            GameOverRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(title),
                TextColor(title_color),
                TextFont::from_font_size(48.0),
            ));
            parent.spawn((
                Text::new(format!("Score: {}  |  Level: {}", fmt_score(score.score), score.level)),
                TextColor(Color::WHITE),
                TextFont::from_font_size(28.0),
            ));
            parent.spawn((
                Text::new(best_line),
                TextColor(Color::srgb(0.6, 0.9, 1.0)),
                TextFont::from_font_size(22.0),
            ));
            parent.spawn((
                Text::new("Press SPACE to return to mode select"),
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                TextFont::from_font_size(22.0),
            ));
        });
}

pub fn despawn_game_over(mut commands: Commands, query: Query<Entity, With<GameOverRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn game_over_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<ScoreBoard>,
    mut board: ResMut<Board>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        reset_game(&mut score, &mut board, 1);
        next_state.set(GameState::ModeSelect);
    }
}

// ── SPRINT COMPLETE ───────────────────────────────────────────────────────────

#[derive(Component)]
pub struct SprintCompleteRoot;

pub fn setup_sprint_complete(
    mut commands: Commands,
    timer: Res<ModeTimer>,
    mut config: ResMut<AppConfig>,
) {
    let elapsed = timer.elapsed;
    let time_str = fmt_time_sprint(elapsed);

    let old_best = config.high_scores.sprint_best;
    let is_new_best = config.try_update_sprint(elapsed);

    let (record_text, record_color) = if is_new_best {
        ("¡NUEVO RÉCORD!".to_string(), Color::srgb(0.2, 1.0, 0.3))
    } else {
        let best_str = fmt_time_sprint(old_best.unwrap_or(elapsed));
        (format!("Mejor: {best_str}"), Color::srgb(0.5, 0.5, 0.5))
    };

    commands
        .spawn((
            SprintCompleteRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("SPRINT COMPLETE!"),
                TextColor(Color::srgb(0.3, 1.0, 0.5)),
                TextFont::from_font_size(48.0),
            ));
            parent.spawn((
                Text::new(format!("Time: {time_str}")),
                TextColor(Color::WHITE),
                TextFont::from_font_size(32.0),
            ));
            parent.spawn((
                Text::new(record_text),
                TextColor(record_color),
                TextFont::from_font_size(24.0),
            ));
            parent.spawn((
                Text::new("Press SPACE to return to mode select"),
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                TextFont::from_font_size(22.0),
            ));
        });
}

pub fn despawn_sprint_complete(
    mut commands: Commands,
    query: Query<Entity, With<SprintCompleteRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn sprint_complete_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<ScoreBoard>,
    mut board: ResMut<Board>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        reset_game(&mut score, &mut board, 1);
        next_state.set(GameState::ModeSelect);
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Format a score number with thousands separators (`1234` → `1.234`).
pub fn fmt_score(n: u32) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    let rem = bytes.len() % 3;
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (i % 3 == rem) {
            out.push('.');
        }
        out.push(b as char);
    }
    out
}

/// Format seconds as `MM:SS.t` (tenths of second).
pub fn fmt_time_sprint(secs: f32) -> String {
    let total_tenths = (secs * 10.0) as u32;
    let tenths = total_tenths % 10;
    let total_secs = total_tenths / 10;
    let s = total_secs % 60;
    let m = total_secs / 60;
    format!("{m:02}:{s:02}.{tenths}")
}

// ── SETTINGS ──────────────────────────────────────────────────────────────────

use bevy::audio::Volume;
use crate::config::keycode_to_str;
use crate::input::PieceAction;
use crate::player::PlayerId;
use crate::state::RebindTarget;

#[derive(Component)]
pub struct SettingsRoot;

/// Which (row, col) is currently highlighted. row 0-6, col 0=P1 1=P2.
#[derive(Resource, Default)]
pub struct SettingsCursor {
    pub row: usize,
    pub col: usize,
}

/// Marker for binding label text entities so we can update them.
#[derive(Component)]
pub struct BindingLabel {
    pub player: PlayerId,
    pub action: PieceAction,
}

/// Marker for the volume bar text in the settings screen.
#[derive(Component)]
pub struct VolumeBar;

const ACTIONS: [(PieceAction, &str); 7] = [
    (PieceAction::MoveLeft,   "Move Left"),
    (PieceAction::MoveRight,  "Move Right"),
    (PieceAction::SoftDrop,   "Soft Drop"),
    (PieceAction::HardDrop,   "Hard Drop"),
    (PieceAction::RotateCW,   "Rotate CW"),
    (PieceAction::RotateCCW,  "Rotate CCW"),
    (PieceAction::Hold,       "Hold"),
];

fn volume_bar_str(volume: f32) -> String {
    let filled = (volume * 10.0).round() as usize;
    let empty = 10usize.saturating_sub(filled);
    let pct = (volume * 100.0).round() as u32;
    format!("Volume: {}{}  {}%", "#".repeat(filled), "-".repeat(empty), pct)
}

fn binding_str(config: &AppConfig, player: PlayerId, action: PieceAction) -> String {
    let b = match player {
        PlayerId::P1 => &config.p1,
        PlayerId::P2 => &config.p2,
    };
    let key = match action {
        PieceAction::MoveLeft  => &b.move_left,
        PieceAction::MoveRight => &b.move_right,
        PieceAction::SoftDrop  => &b.soft_drop,
        PieceAction::HardDrop  => &b.hard_drop,
        PieceAction::RotateCW  => &b.rotate_cw,
        PieceAction::RotateCCW => &b.rotate_ccw,
        PieceAction::Hold      => &b.hold,
    };
    format!("[{key}]")
}

pub fn setup_settings(mut commands: Commands, config: Res<AppConfig>) {
    commands.insert_resource(SettingsCursor::default());

    commands
        .spawn((
            SettingsRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.92)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("SETTINGS"),
                TextColor(Color::WHITE),
                TextFont::from_font_size(40.0),
            ));

            // Header row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(40.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new("Action"),
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                        TextFont::from_font_size(18.0),
                        Node { width: Val::Px(140.0), ..default() },
                    ));
                    row.spawn((
                        Text::new("Player 1"),
                        TextColor(Color::srgb(0.4, 0.65, 1.0)),
                        TextFont::from_font_size(18.0),
                        Node { width: Val::Px(160.0), ..default() },
                    ));
                    row.spawn((
                        Text::new("Player 2"),
                        TextColor(Color::srgb(1.0, 0.75, 0.55)),
                        TextFont::from_font_size(18.0),
                        Node { width: Val::Px(160.0), ..default() },
                    ));
                });

            // Action rows
            for (action, name) in ACTIONS {
                let p1_str = binding_str(&config, PlayerId::P1, action);
                let p2_str = binding_str(&config, PlayerId::P2, action);

                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(40.0),
                        ..default()
                    })
                    .with_children(|row| {
                        // Action label
                        row.spawn((
                            Text::new(name),
                            TextColor(Color::srgb(0.85, 0.85, 0.85)),
                            TextFont::from_font_size(18.0),
                            Node { width: Val::Px(140.0), ..default() },
                        ));
                        // P1 binding
                        row.spawn((
                            BindingLabel { player: PlayerId::P1, action },
                            Text::new(p1_str),
                            TextColor(Color::srgb(0.8, 0.8, 0.8)),
                            TextFont::from_font_size(18.0),
                            Node { width: Val::Px(160.0), ..default() },
                        ));
                        // P2 binding
                        row.spawn((
                            BindingLabel { player: PlayerId::P2, action },
                            Text::new(p2_str),
                            TextColor(Color::srgb(0.8, 0.8, 0.8)),
                            TextFont::from_font_size(18.0),
                            Node { width: Val::Px(160.0), ..default() },
                        ));
                    });
            }

            // Volume control row
            parent.spawn((
                VolumeBar,
                Text::new(volume_bar_str(config.volume)),
                TextColor(Color::srgb(0.8, 0.85, 1.0)),
                TextFont::from_font_size(18.0),
            ));

            parent.spawn((
                Text::new("Up/Down: navigate  |  Left/Right: select column or adjust volume  |  Enter: remap key  |  ESC: save & return"),
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                TextFont::from_font_size(14.0),
            ));
        });
}

pub fn despawn_settings(mut commands: Commands, query: Query<Entity, With<SettingsRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<SettingsCursor>();
}

const VOLUME_ROW: usize = ACTIONS.len(); // row 7

/// Updates the highlight colors of binding labels based on cursor + rebind state.
pub fn update_settings_highlight(
    cursor: Option<Res<SettingsCursor>>,
    rebind: Res<RebindTarget>,
    mut labels: Query<(&BindingLabel, &mut TextColor)>,
    mut volume_bar_color: Query<&mut TextColor, (With<VolumeBar>, Without<BindingLabel>)>,
) {
    let Some(cursor) = cursor else { return };

    // Highlight volume bar
    for mut color in &mut volume_bar_color {
        *color = if cursor.row == VOLUME_ROW {
            TextColor(Color::srgb(1.0, 0.9, 0.1))
        } else {
            TextColor(Color::srgb(0.8, 0.85, 1.0))
        };
    }

    if cursor.row >= VOLUME_ROW {
        // Deselect all binding labels
        for (_, mut color) in &mut labels {
            *color = TextColor(Color::srgb(0.8, 0.8, 0.8));
        }
        return;
    }

    let action_at_row = ACTIONS[cursor.row].0;

    for (label, mut color) in &mut labels {
        let is_selected_row = label.action == action_at_row;
        let is_selected_col =
            (cursor.col == 0 && label.player == PlayerId::P1) ||
            (cursor.col == 1 && label.player == PlayerId::P2);
        let is_pending = rebind.pending.map_or(false, |(p, a)| p == label.player && a == label.action);

        *color = if is_pending {
            TextColor(Color::srgb(1.0, 0.5, 0.0))
        } else if is_selected_row && is_selected_col {
            TextColor(Color::srgb(1.0, 0.9, 0.1))
        } else {
            TextColor(Color::srgb(0.8, 0.8, 0.8))
        };
    }
}

pub fn settings_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    cursor: Option<ResMut<SettingsCursor>>,
    mut rebind: ResMut<RebindTarget>,
    mut config: ResMut<AppConfig>,
    mut next_state: ResMut<NextState<GameState>>,
    mut labels: Query<(&BindingLabel, &mut Text)>,
    mut volume_bar: Query<&mut Text, (With<VolumeBar>, Without<BindingLabel>)>,
    mut sink: Query<&mut bevy::audio::AudioSink, With<crate::audio::BgMusic>>,
) {
    let Some(mut cursor) = cursor else { return };

    // If waiting for a key capture
    if let Some((player, action)) = rebind.pending {
        let captured = keyboard.get_just_pressed().find(|&&kc| kc != KeyCode::Escape).copied();
        if let Some(kc) = captured {
            // Only accept keys that are in the whitelist so the binding
            // round-trips correctly through the JSON config.
            if let Some(key_str) = keycode_to_str(kc) {
                {
                    let b = match player {
                        PlayerId::P1 => &mut config.p1,
                        PlayerId::P2 => &mut config.p2,
                    };
                    let field = match action {
                        PieceAction::MoveLeft  => &mut b.move_left,
                        PieceAction::MoveRight => &mut b.move_right,
                        PieceAction::SoftDrop  => &mut b.soft_drop,
                        PieceAction::HardDrop  => &mut b.hard_drop,
                        PieceAction::RotateCW  => &mut b.rotate_cw,
                        PieceAction::RotateCCW => &mut b.rotate_ccw,
                        PieceAction::Hold      => &mut b.hold,
                    };
                    *field = key_str.to_string();
                }
                for (label, mut text) in &mut labels {
                    if label.player == player && label.action == action {
                        **text = format!("[{key_str}]");
                    }
                }
                rebind.pending = None;
            }
            // If the key is not in the whitelist, stay in capture mode and wait
            // for a recognised key (the user will see the row still blinking).
        } else if keyboard.just_pressed(KeyCode::Escape) {
            rebind.pending = None;
        }
        return;
    }

    // Navigation: up/down always move between rows
    if keyboard.just_pressed(KeyCode::ArrowUp) && cursor.row > 0 {
        cursor.row -= 1;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) && cursor.row < VOLUME_ROW {
        cursor.row += 1;
    }

    if cursor.row == VOLUME_ROW {
        // Left/Right adjust volume
        let vol_changed = if keyboard.just_pressed(KeyCode::ArrowLeft) {
            config.volume = ((config.volume - 0.1) * 10.0).round() / 10.0;
            config.volume = config.volume.clamp(0.0, 1.0);
            true
        } else if keyboard.just_pressed(KeyCode::ArrowRight) {
            config.volume = ((config.volume + 0.1) * 10.0).round() / 10.0;
            config.volume = config.volume.clamp(0.0, 1.0);
            true
        } else {
            false
        };

        if vol_changed {
            for mut s in sink.iter_mut() {
                s.set_volume(Volume::Linear(config.volume));
            }
            for mut text in &mut volume_bar {
                **text = volume_bar_str(config.volume);
            }
            config.save();
        }
    } else {
        // Left/Right switch P1/P2 column
        if keyboard.just_pressed(KeyCode::ArrowLeft) {
            cursor.col = 0;
        }
        if keyboard.just_pressed(KeyCode::ArrowRight) {
            cursor.col = 1;
        }

        // Enter: start capture
        if keyboard.just_pressed(KeyCode::Enter) {
            let player = if cursor.col == 0 { PlayerId::P1 } else { PlayerId::P2 };
            let action = ACTIONS[cursor.row].0;
            rebind.pending = Some((player, action));
        }
    }

    // ESC: save and return
    if keyboard.just_pressed(KeyCode::Escape) {
        config.save();
        next_state.set(GameState::ModeSelect);
    }
}

/// Toggle mute on M key press. Active in Menu and Paused states.
pub fn global_mute_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<AppConfig>,
    mut sink: Query<&mut bevy::audio::AudioSink, With<crate::audio::BgMusic>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyM) {
        return;
    }
    if config.volume > 0.0 {
        // Mute: save current volume and set to 0
        config.volume_before_mute = Some(config.volume);
        config.volume = 0.0;
    } else {
        // Unmute: restore saved volume (or default to 1.0)
        config.volume = config.volume_before_mute.unwrap_or(1.0);
        config.volume_before_mute = None;
    }
    for mut s in sink.iter_mut() {
        s.set_volume(Volume::Linear(config.volume));
    }
    config.save();
}
