use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::config::{str_to_keycode, AppConfig};
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
    map.insert(PieceAction::MoveLeft,  str_to_keycode(&bindings.move_left));
    map.insert(PieceAction::MoveRight, str_to_keycode(&bindings.move_right));
    map.insert(PieceAction::SoftDrop,  str_to_keycode(&bindings.soft_drop));
    map.insert(PieceAction::HardDrop,  str_to_keycode(&bindings.hard_drop));
    map.insert(PieceAction::RotateCW,  str_to_keycode(&bindings.rotate_cw));
    map.insert(PieceAction::RotateCCW, str_to_keycode(&bindings.rotate_ccw));
    map.insert(PieceAction::Hold,      str_to_keycode(&bindings.hold));
    map
}
