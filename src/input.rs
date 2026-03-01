use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::config::{str_to_keycode, AppConfig, PlayerBindings};
use crate::player::PlayerId;

#[derive(Actionlike, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum PieceAction {
    MoveLeft,
    MoveRight,
    SoftDrop,
    HardDrop,
    RotateCW,
    RotateCCW,
    Hold,
}

pub fn input_map_for(player: PlayerId, config: &AppConfig) -> InputMap<PieceAction> {
    let mut map = InputMap::default();
    let bindings = match player {
        PlayerId::P1 => &config.p1,
        PlayerId::P2 => &config.p2,
    };
    let defaults = match player {
        PlayerId::P1 => PlayerBindings::p1_defaults(),
        PlayerId::P2 => PlayerBindings::p2_defaults(),
    };
    // str_to_keycode returns None for unrecognised key names; fall back to the
    // action's default binding so the game stays playable.
    let kc = |s: &str, d: &str| str_to_keycode(s).unwrap_or_else(|| str_to_keycode(d).unwrap());
    map.insert(
        PieceAction::MoveLeft,
        kc(&bindings.move_left, &defaults.move_left),
    );
    map.insert(
        PieceAction::MoveRight,
        kc(&bindings.move_right, &defaults.move_right),
    );
    map.insert(
        PieceAction::SoftDrop,
        kc(&bindings.soft_drop, &defaults.soft_drop),
    );
    map.insert(
        PieceAction::HardDrop,
        kc(&bindings.hard_drop, &defaults.hard_drop),
    );
    map.insert(
        PieceAction::RotateCW,
        kc(&bindings.rotate_cw, &defaults.rotate_cw),
    );
    map.insert(
        PieceAction::RotateCCW,
        kc(&bindings.rotate_ccw, &defaults.rotate_ccw),
    );
    map.insert(PieceAction::Hold, kc(&bindings.hold, &defaults.hold));
    map
}
