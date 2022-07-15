use bevy::prelude::Gamepad;
use leafwing_input_manager::{axislike::VirtualDPad, prelude::InputMap, user_input::InputKind};
use serde::{Deserialize, Serialize};

use crate::input::PlayerAction;

/// Global settings, stored and accessed through [`crate::platform::Storage`]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Settings {
    pub player_controls: PlayerControlMethods,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PlayerControlMethods {
    pub gamepad: PlayerControls,
    pub keyboard1: PlayerControls,
    pub keyboard2: PlayerControls,
}

impl PlayerControlMethods {
    pub(crate) fn get_input_map(&self, player_idx: usize) -> InputMap<PlayerAction> {
        let mut input_map = InputMap::default();

        input_map.set_gamepad(Gamepad(player_idx));

        let mut add_controls = |ctrls: &PlayerControls| {
            input_map.insert(ctrls.movement.clone(), PlayerAction::Move);
            input_map.insert(ctrls.flop_attack, PlayerAction::FlopAttack);
            input_map.insert(ctrls.shoot, PlayerAction::Shoot);
            input_map.insert(ctrls.throw, PlayerAction::Throw);
        };

        add_controls(&self.gamepad);

        match player_idx {
            0 => add_controls(&self.keyboard1),
            1 => add_controls(&self.keyboard2),
            _ => (),
        }

        input_map
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PlayerControls {
    pub movement: VirtualDPad,
    pub flop_attack: InputKind,
    pub throw: InputKind,
    pub shoot: InputKind,
}

impl Settings {
    pub const STORAGE_KEY: &'static str = "settings";
}
