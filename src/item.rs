use bevy::{
    math::{Vec2, Vec3},
    prelude::{AssetServer, Bundle, Commands, Component, EventReader, Handle, Res, Transform},
    transform::TransformBundle,
};

use crate::{
    animation::Facing,
    attack::ThrownWeapon,
    consts::{self, ITEM_LAYER},
    metadata::{ItemMeta, ItemSpawnMeta},
};

#[derive(Component)]
pub struct Item;

#[derive(Bundle)]
pub struct ItemSpawnBundle {
    item_meta_handle: Handle<ItemMeta>,
    #[bundle]
    transform_bundle: TransformBundle,
}

impl ItemSpawnBundle {
    pub fn new(item_spawn_meta: &ItemSpawnMeta) -> Self {
        let item_meta_handle = item_spawn_meta.item_handle.clone();

        let ground_offset = Vec3::new(0.0, consts::GROUND_Y, ITEM_LAYER);
        let transform_bundle = TransformBundle::from_transform(Transform::from_translation(
            item_spawn_meta.location + ground_offset,
        ));

        Self {
            item_meta_handle,
            transform_bundle,
        }
    }
}

pub struct ThrowItemEvent {
    pub position: Vec2,
    pub facing: Facing,
}

pub fn spawn_throwable_items(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut event: EventReader<ThrowItemEvent>,
) {
    for ev in event.iter() {
        let angles = match ev.facing {
            Facing::Left => (90. - consts::THROW_ITEM_ANGLE_OFFSET, 180.),
            Facing::Right => (90. + consts::THROW_ITEM_ANGLE_OFFSET, 0.),
        };

        let thrown_weapon = ThrownWeapon::new(angles, ev, &asset_server);

        commands.spawn_bundle(thrown_weapon);
    }
}
