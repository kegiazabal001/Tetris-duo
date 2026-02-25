use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerBindings {
    pub move_left:  String,
    pub move_right: String,
    pub soft_drop:  String,
    pub hard_drop:  String,
    pub rotate_cw:  String,
    pub rotate_ccw: String,
    pub hold:       String,
}

impl PlayerBindings {
    pub fn p1_defaults() -> Self {
        Self {
            move_left:  "KeyA".into(),
            move_right: "KeyD".into(),
            soft_drop:  "KeyS".into(),
            hard_drop:  "Space".into(),
            rotate_cw:  "KeyW".into(),
            rotate_ccw: "KeyQ".into(),
            hold:       "ShiftLeft".into(),
        }
    }

    pub fn p2_defaults() -> Self {
        Self {
            move_left:  "ArrowLeft".into(),
            move_right: "ArrowRight".into(),
            soft_drop:  "ArrowDown".into(),
            hard_drop:  "Enter".into(),
            rotate_cw:  "ArrowUp".into(),
            rotate_ccw: "ControlRight".into(),
            hold:       "ShiftRight".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HighScores {
    pub endless:     u32,
    pub sprint_best: Option<f32>,
    pub ultra_best:  u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct AppConfig {
    pub p1:          PlayerBindings,
    pub p2:          PlayerBindings,
    pub high_scores: HighScores,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            p1:          PlayerBindings::p1_defaults(),
            p2:          PlayerBindings::p2_defaults(),
            high_scores: HighScores::default(),
        }
    }
}

pub fn config_path() -> PathBuf {
    // Use the standard home-dir resolution instead of interpolating $HOME
    // directly, which would allow path traversal if the env var is attacker-
    // controlled (e.g. HOME=/tmp/../../etc).
    let dir = home_config_dir();
    let _ = std::fs::create_dir_all(&dir);
    dir.join("settings.json")
}

fn home_config_dir() -> PathBuf {
    // std::env::home_dir is deprecated but still correct on Linux/macOS.
    // It resolves via passwd on Unix rather than trusting $HOME blindly.
    #[allow(deprecated)]
    let home = std::env::home_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".config").join("tetris-duo")
}

impl AppConfig {
    pub fn load() -> Self {
        let path = config_path();
        if let Ok(contents) = std::fs::read_to_string(&path) {
            match serde_json::from_str(&contents) {
                Ok(config) => return config,
                Err(e) => eprintln!("Advertencia: no se pudo leer settings.json ({e}), usando valores por defecto"),
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = config_path();
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }
}

/// PreStartup system: loads AppConfig from disk and seeds ScoreBoard's high score.
pub fn load_config(
    mut commands: Commands,
    mut score: ResMut<crate::scoring::ScoreBoard>,
) {
    let config = AppConfig::load();
    score.high_score = config.high_scores.endless;
    commands.insert_resource(config);
}

/// Converts a key name string (as stored in JSON) to the corresponding `KeyCode`.
/// Returns `None` for strings that are not in the known whitelist so callers
/// can fall back to the action's default binding instead of silently using KeyA.
pub fn str_to_keycode(s: &str) -> Option<KeyCode> {
    Some(match s {
        // Letters
        "KeyA" => KeyCode::KeyA,
        "KeyB" => KeyCode::KeyB,
        "KeyC" => KeyCode::KeyC,
        "KeyD" => KeyCode::KeyD,
        "KeyE" => KeyCode::KeyE,
        "KeyF" => KeyCode::KeyF,
        "KeyG" => KeyCode::KeyG,
        "KeyH" => KeyCode::KeyH,
        "KeyI" => KeyCode::KeyI,
        "KeyJ" => KeyCode::KeyJ,
        "KeyK" => KeyCode::KeyK,
        "KeyL" => KeyCode::KeyL,
        "KeyM" => KeyCode::KeyM,
        "KeyN" => KeyCode::KeyN,
        "KeyO" => KeyCode::KeyO,
        "KeyP" => KeyCode::KeyP,
        "KeyQ" => KeyCode::KeyQ,
        "KeyR" => KeyCode::KeyR,
        "KeyS" => KeyCode::KeyS,
        "KeyT" => KeyCode::KeyT,
        "KeyU" => KeyCode::KeyU,
        "KeyV" => KeyCode::KeyV,
        "KeyW" => KeyCode::KeyW,
        "KeyX" => KeyCode::KeyX,
        "KeyY" => KeyCode::KeyY,
        "KeyZ" => KeyCode::KeyZ,
        // Digits
        "Digit0" => KeyCode::Digit0,
        "Digit1" => KeyCode::Digit1,
        "Digit2" => KeyCode::Digit2,
        "Digit3" => KeyCode::Digit3,
        "Digit4" => KeyCode::Digit4,
        "Digit5" => KeyCode::Digit5,
        "Digit6" => KeyCode::Digit6,
        "Digit7" => KeyCode::Digit7,
        "Digit8" => KeyCode::Digit8,
        "Digit9" => KeyCode::Digit9,
        // Arrows
        "ArrowLeft"  => KeyCode::ArrowLeft,
        "ArrowRight" => KeyCode::ArrowRight,
        "ArrowUp"    => KeyCode::ArrowUp,
        "ArrowDown"  => KeyCode::ArrowDown,
        // Special keys
        "Space"        => KeyCode::Space,
        "Enter"        => KeyCode::Enter,
        "Escape"       => KeyCode::Escape,
        "Backspace"    => KeyCode::Backspace,
        "Tab"          => KeyCode::Tab,
        "ShiftLeft"    => KeyCode::ShiftLeft,
        "ShiftRight"   => KeyCode::ShiftRight,
        "ControlLeft"  => KeyCode::ControlLeft,
        "ControlRight" => KeyCode::ControlRight,
        "AltLeft"      => KeyCode::AltLeft,
        "AltRight"     => KeyCode::AltRight,
        // Punctuation
        "Comma"        => KeyCode::Comma,
        "Period"       => KeyCode::Period,
        "Slash"        => KeyCode::Slash,
        "Backslash"    => KeyCode::Backslash,
        "Semicolon"    => KeyCode::Semicolon,
        "Quote"        => KeyCode::Quote,
        "BracketLeft"  => KeyCode::BracketLeft,
        "BracketRight" => KeyCode::BracketRight,
        "Minus"        => KeyCode::Minus,
        "Equal"        => KeyCode::Equal,
        "Backquote"    => KeyCode::Backquote,
        // F-keys
        "F1"  => KeyCode::F1,
        "F2"  => KeyCode::F2,
        "F3"  => KeyCode::F3,
        "F4"  => KeyCode::F4,
        "F5"  => KeyCode::F5,
        "F6"  => KeyCode::F6,
        "F7"  => KeyCode::F7,
        "F8"  => KeyCode::F8,
        "F9"  => KeyCode::F9,
        "F10" => KeyCode::F10,
        "F11" => KeyCode::F11,
        "F12" => KeyCode::F12,
        // Numpad
        "Numpad0"        => KeyCode::Numpad0,
        "Numpad1"        => KeyCode::Numpad1,
        "Numpad2"        => KeyCode::Numpad2,
        "Numpad3"        => KeyCode::Numpad3,
        "Numpad4"        => KeyCode::Numpad4,
        "Numpad5"        => KeyCode::Numpad5,
        "Numpad6"        => KeyCode::Numpad6,
        "Numpad7"        => KeyCode::Numpad7,
        "Numpad8"        => KeyCode::Numpad8,
        "Numpad9"        => KeyCode::Numpad9,
        "NumpadAdd"      => KeyCode::NumpadAdd,
        "NumpadSubtract" => KeyCode::NumpadSubtract,
        "NumpadMultiply" => KeyCode::NumpadMultiply,
        "NumpadDivide"   => KeyCode::NumpadDivide,
        "NumpadEnter"    => KeyCode::NumpadEnter,
        "NumpadDecimal"  => KeyCode::NumpadDecimal,
        // Navigation
        "Home"     => KeyCode::Home,
        "End"      => KeyCode::End,
        "PageUp"   => KeyCode::PageUp,
        "PageDown" => KeyCode::PageDown,
        "Insert"   => KeyCode::Insert,
        "Delete"   => KeyCode::Delete,
        _ => return None,
    })
}

/// Converts a `KeyCode` to the string name used in the JSON config.
/// Returns `None` if the key is not in the whitelist recognised by `str_to_keycode`,
/// preventing rebinds that would silently break on the next load.
pub fn keycode_to_str(kc: KeyCode) -> Option<String> {
    let s = format!("{kc:?}");
    // Round-trip check: only accept keys that survive the str→keycode conversion.
    if str_to_keycode(&s).is_some() {
        Some(s)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_round_trips_json() {
        let original = AppConfig::default();
        let json = serde_json::to_string(&original).unwrap();
        let restored: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.p1.move_left, original.p1.move_left);
        assert_eq!(restored.p2.hard_drop, original.p2.hard_drop);
        assert_eq!(restored.high_scores.endless, original.high_scores.endless);
    }

    #[test]
    fn sprint_best_none_by_default() {
        assert!(HighScores::default().sprint_best.is_none());
    }
}
