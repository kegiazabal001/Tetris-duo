use bevy::prelude::*;

use crate::board::Board;
use crate::config::AppConfig;
use crate::modes::{ModeTimer, SPRINT_GOAL, ULTRA_DURATION};
use crate::scoring::ScoreBoard;
use crate::state::{GameState, QuitToMenu, SelectedMode};

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
        **text = format!("Score: {}", score.score);
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
        **text = format!("Best: {}", score.high_score);
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
            let color =
                if remaining < 30.0 { Color::srgb(1.0, 0.2, 0.2) } else { Color::WHITE };
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
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("TETRIS DUO"),
                TextColor(Color::WHITE),
                TextFont::from_font_size(48.0),
            ));
            parent.spawn((
                Text::new("Cooperative — clear lines together!"),
                TextColor(Color::srgb(0.6, 0.9, 0.6)),
                TextFont::from_font_size(18.0),
            ));
            parent.spawn((
                Text::new("Press SPACE to continue"),
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                TextFont::from_font_size(24.0),
            ));
            parent.spawn((
                Text::new("P1: A/D move  W rotate  S soft-drop  Q/E rotate  LShift hold"),
                TextColor(Color::srgb(0.4, 0.65, 1.0)),
                TextFont::from_font_size(15.0),
            ));
            parent.spawn((
                Text::new("P2: </> move  Up/- rotate  Dn soft-drop  RCtrl rotate  RShift hold"),
                TextColor(Color::srgb(1.0, 0.75, 0.55)),
                TextFont::from_font_size(15.0),
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

pub fn setup_mode_select(mut commands: Commands, config: Res<AppConfig>) {
    let endless_best = format!("Best: {} pts", config.high_scores.endless);
    let sprint_best = match config.high_scores.sprint_best {
        Some(secs) => format!("Best: {}", fmt_time_sprint(secs)),
        None => "Best: —".to_string(),
    };
    let ultra_best = format!("Best: {} pts", config.high_scores.ultra_best);

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
                Text::new(format!("1  ENDLESS  —  {endless_best}")),
                TextColor(Color::srgb(0.6, 0.9, 1.0)),
                TextFont::from_font_size(24.0),
            ));
            // Sprint
            parent.spawn((
                Text::new(format!("2  SPRINT  (40 lines)  —  {sprint_best}")),
                TextColor(Color::srgb(0.4, 1.0, 0.6)),
                TextFont::from_font_size(24.0),
            ));
            // Ultra
            parent.spawn((
                Text::new(format!("3  ULTRA  (2 min)  —  {ultra_best}")),
                TextColor(Color::srgb(1.0, 0.7, 0.3)),
                TextFont::from_font_size(24.0),
            ));

            parent.spawn((
                Text::new("S — Settings"),
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

pub fn mode_select_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut selected_mode: ResMut<SelectedMode>,
    mut score: ResMut<ScoreBoard>,
    mut board: ResMut<Board>,
) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        *selected_mode = SelectedMode::Endless;
        score.reset_preserving_high_score();
        *board = Board::default();
        next_state.set(GameState::Playing);
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        *selected_mode = SelectedMode::Sprint;
        score.reset_preserving_high_score();
        *board = Board::default();
        next_state.set(GameState::Playing);
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        *selected_mode = SelectedMode::Ultra;
        score.reset_preserving_high_score();
        *board = Board::default();
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
                Text::new("Q - quit to menu"),
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
    mut quit_to_menu: ResMut<QuitToMenu>,
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
            quit_to_menu.0 = true;
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
            format!("Best: {} pts", config.high_scores.endless.max(score.score))
        }
        SelectedMode::Sprint => match config.high_scores.sprint_best {
            Some(secs) => format!("Best: {}", fmt_time_sprint(secs)),
            None => "Best: —".to_string(),
        },
        SelectedMode::Ultra => {
            format!("Best: {} pts", config.high_scores.ultra_best.max(score.score))
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
                Text::new(format!("Score: {} | Level: {}", score.score, score.level)),
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
        score.reset_preserving_high_score();
        *board = Board::default();
        next_state.set(GameState::ModeSelect);
    }
}

// ── SPRINT COMPLETE ───────────────────────────────────────────────────────────

#[derive(Component)]
pub struct SprintCompleteRoot;

pub fn setup_sprint_complete(mut commands: Commands, timer: Res<ModeTimer>) {
    let time_str = fmt_time_sprint(timer.elapsed);
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
        score.reset_preserving_high_score();
        *board = Board::default();
        next_state.set(GameState::ModeSelect);
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Format seconds as `MM:SS.t` (tenths of second).
pub fn fmt_time_sprint(secs: f32) -> String {
    let total_tenths = (secs * 10.0) as u32;
    let tenths = total_tenths % 10;
    let total_secs = total_tenths / 10;
    let s = total_secs % 60;
    let m = total_secs / 60;
    format!("{m:02}:{s:02}.{tenths}")
}
