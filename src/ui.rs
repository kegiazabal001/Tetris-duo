use bevy::prelude::*;

use crate::board::Board;
use crate::scoring::ScoreBoard;
use crate::state::{GameState, QuitToMenu};

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
pub struct MenuRoot;

#[derive(Component)]
pub struct PauseRoot;

#[derive(Component)]
pub struct GameOverRoot;

pub fn setup_hud(mut commands: Commands) {
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
    mut score_q: Query<
        &mut Text,
        (With<ScoreText>, Without<LevelText>, Without<ComboText>, Without<HighScoreText>),
    >,
    mut level_q: Query<
        &mut Text,
        (With<LevelText>, Without<ScoreText>, Without<ComboText>, Without<HighScoreText>),
    >,
    mut combo_q: Query<
        &mut Text,
        (With<ComboText>, Without<ScoreText>, Without<LevelText>, Without<HighScoreText>),
    >,
    mut hs_q: Query<
        &mut Text,
        (With<HighScoreText>, Without<ScoreText>, Without<LevelText>, Without<ComboText>),
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
}

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
                Text::new("Cooperativo - completad lineas juntos!"),
                TextColor(Color::srgb(0.6, 0.9, 0.6)),
                TextFont::from_font_size(18.0),
            ));
            parent.spawn((
                Text::new("Press SPACE to start"),
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                TextFont::from_font_size(24.0),
            ));
            parent.spawn((
                Text::new("P1: A/D move  W hard-drop  S soft-drop  Q/E rotate  LShift hold"),
                TextColor(Color::srgb(0.4, 0.65, 1.0)),
                TextFont::from_font_size(15.0),
            ));
            parent.spawn((
                Text::new("P2: </> move  Up hard-drop  Dn soft-drop  ,/. rotate  RShift hold"),
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

pub fn menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
}

pub fn setup_game_over(mut commands: Commands, score: Res<ScoreBoard>) {
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
                Text::new("GAME OVER"),
                TextColor(Color::srgb(1.0, 0.3, 0.3)),
                TextFont::from_font_size(48.0),
            ));
            parent.spawn((
                Text::new(format!("Score: {} | Level: {}", score.score, score.level)),
                TextColor(Color::WHITE),
                TextFont::from_font_size(28.0),
            ));
            parent.spawn((
                Text::new(format!("Best: {}", score.high_score.max(score.score))),
                TextColor(Color::srgb(0.6, 0.9, 1.0)),
                TextFont::from_font_size(22.0),
            ));
            parent.spawn((
                Text::new("Press SPACE to restart"),
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                TextFont::from_font_size(24.0),
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
        next_state.set(GameState::Playing);
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
