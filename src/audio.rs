use bevy::audio::Volume;
use bevy::prelude::*;

use crate::config::AppConfig;
use crate::piece::TSpinType;
use crate::player::{GameOverEvent, LinesCleared, PieceRotated};
use crate::scoring::LevelUpEvent;

#[derive(Resource)]
pub struct AudioAssets {
    pub rotate: Handle<AudioSource>,
    pub land: Handle<AudioSource>,
    pub line_clear: Handle<AudioSource>,
    pub tetris: Handle<AudioSource>,
    pub level_up: Handle<AudioSource>,
    pub game_over: Handle<AudioSource>,
    pub music: Handle<AudioSource>,
    pub game_music: Handle<AudioSource>,
}

/// Marker component for the background music entity.
#[derive(Component)]
pub struct BgMusic;

/// Marker component for the in-game music entity.
#[derive(Component)]
pub struct GameMusic;

fn play(commands: &mut Commands, handle: Handle<AudioSource>) {
    commands.spawn((AudioPlayer::new(handle), PlaybackSettings::ONCE));
}

impl FromWorld for AudioAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        AudioAssets {
            rotate: asset_server.load("audio/rotate.ogg"),
            land: asset_server.load("audio/land.ogg"),
            line_clear: asset_server.load("audio/line_clear.ogg"),
            tetris: asset_server.load("audio/tetris.ogg"),
            level_up: asset_server.load("audio/level_up.ogg"),
            game_over: asset_server.load("audio/game_over.ogg"),
            music: asset_server.load("audio/retro_menu_groove.ogg"),
            game_music: asset_server.load("audio/Casual_8bit.ogg"),
        }
    }
}

pub fn start_bg_music(
    mut commands: Commands,
    audio: Res<AudioAssets>,
    config: Res<AppConfig>,
    query_bg: Query<Entity, With<BgMusic>>,
) {
    if !query_bg.is_empty() {
        return;
    }
    commands.spawn((
        AudioPlayer::new(audio.music.clone()),
        PlaybackSettings {
            volume: Volume::Linear(config.volume),
            mode: bevy::audio::PlaybackMode::Loop,
            ..default()
        },
        BgMusic,
    ));
}

pub fn stop_bg_music(mut commands: Commands, query: Query<Entity, With<BgMusic>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn start_game_music(
    mut commands: Commands,
    audio: Res<AudioAssets>,
    config: Res<AppConfig>,
    query_game: Query<Entity, With<GameMusic>>,
) {
    if !query_game.is_empty() {
        return;
    }
    commands.spawn((
        AudioPlayer::new(audio.game_music.clone()),
        PlaybackSettings {
            volume: Volume::Linear(config.volume),
            mode: bevy::audio::PlaybackMode::Loop,
            ..default()
        },
        GameMusic,
    ));
}

pub fn stop_game_music(mut commands: Commands, query: Query<Entity, With<GameMusic>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Updates the volume of the background music sink to match AppConfig.
pub fn set_bg_volume(mut query: Query<&mut AudioSink, With<BgMusic>>, config: Res<AppConfig>) {
    for mut sink in query.iter_mut() {
        sink.set_volume(Volume::Linear(config.volume));
    }
}

pub fn pause_bg_music(query: Query<&AudioSink, With<BgMusic>>) {
    for sink in query.iter() {
        sink.pause();
    }
}

pub fn resume_bg_music(query: Query<&AudioSink, With<BgMusic>>) {
    for sink in query.iter() {
        sink.play();
    }
}

pub fn play_piece_sounds(
    mut commands: Commands,
    audio: Res<AudioAssets>,
    mut ev_locked: EventReader<LinesCleared>,
) {
    for event in ev_locked.read() {
        let handle = if event.count >= 4 || matches!(event.t_spin, TSpinType::Full) {
            audio.tetris.clone()
        } else if event.count > 0 {
            audio.line_clear.clone()
        } else {
            audio.land.clone()
        };
        play(&mut commands, handle);
    }
}

pub fn play_rotate_sound(
    mut commands: Commands,
    audio: Res<AudioAssets>,
    mut ev: EventReader<PieceRotated>,
) {
    for _ in ev.read() {
        play(&mut commands, audio.rotate.clone());
    }
}

pub fn play_level_up_sound(
    mut commands: Commands,
    audio: Res<AudioAssets>,
    mut ev: EventReader<LevelUpEvent>,
) {
    for _ in ev.read() {
        play(&mut commands, audio.level_up.clone());
    }
}

pub fn play_game_over_sound(
    mut commands: Commands,
    audio: Res<AudioAssets>,
    mut ev: EventReader<GameOverEvent>,
) {
    for _ in ev.read() {
        play(&mut commands, audio.game_over.clone());
    }
}
