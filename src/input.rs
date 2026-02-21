use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

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

pub fn input_map_for(player: PlayerId) -> InputMap<PieceAction> {
    let mut map = InputMap::default();
    match player {
        PlayerId::P1 => {
            map.insert(PieceAction::MoveLeft, KeyCode::KeyA);
            map.insert(PieceAction::MoveRight, KeyCode::KeyD);
            map.insert(PieceAction::SoftDrop, KeyCode::KeyS);
            map.insert(PieceAction::HardDrop, KeyCode::Space);
            map.insert(PieceAction::RotateCCW, KeyCode::KeyQ);
            map.insert(PieceAction::RotateCW, KeyCode::KeyW);
            map.insert(PieceAction::Hold, KeyCode::ShiftLeft);
        }
        PlayerId::P2 => {
            map.insert(PieceAction::MoveLeft, KeyCode::ArrowLeft);
            map.insert(PieceAction::MoveRight, KeyCode::ArrowRight);
            map.insert(PieceAction::SoftDrop, KeyCode::ArrowDown);
            map.insert(PieceAction::HardDrop, KeyCode::Enter);
            map.insert(PieceAction::RotateCCW, KeyCode::Comma);
            map.insert(PieceAction::RotateCW, KeyCode::ArrowUp);
            map.insert(PieceAction::Hold, KeyCode::ShiftRight);
        }
    }
    map
}
