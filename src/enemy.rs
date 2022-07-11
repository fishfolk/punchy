use bevy::prelude::*;

use crate::animation::Facing;

#[derive(Component)]
pub struct Enemy;

#[derive(Bundle)]
pub struct EnemyBundle {
    enemy: Enemy,
    facing: Facing,
}

impl Default for EnemyBundle {
    fn default() -> Self {
        EnemyBundle {
            enemy: Enemy,
            facing: Facing::Left,
        }
    }
}
