use bevy::prelude::*;

use crate::{
    animation::Facing,
    consts,
    metadata::{FighterMeta, FighterSpawnMeta},
};

#[derive(Component)]
pub struct Enemy;

#[derive(Bundle)]
pub struct EnemyBundle {
    enemy: Enemy,
    facing: Facing,
    #[bundle]
    transform_bundle: TransformBundle,
    fighter_handle: Handle<FighterMeta>,
}

impl EnemyBundle {
    pub fn new(enemy_meta: &FighterSpawnMeta) -> Self {
        let ground_offset = Vec3::new(0.0, consts::GROUND_Y, 0.0);
        let enemy_pos = enemy_meta.location + ground_offset;

        let transform_bundle =
            TransformBundle::from_transform(Transform::from_translation(enemy_pos));

        let fighter_handle = enemy_meta.fighter_handle.clone();

        EnemyBundle {
            enemy: Enemy,
            facing: Facing::Left,
            transform_bundle,
            fighter_handle,
        }
    }
}
