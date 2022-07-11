use bevy::prelude::*;

use crate::animation::Facing;

#[derive(Component)]
pub struct Player;

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    facing: Facing,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        PlayerBundle {
            player: Player,
            facing: Facing::Right,
        }
    }
}
