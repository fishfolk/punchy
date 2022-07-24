use bevy::{
    ecs::system::EntityCommands,
    math::Vec3,
    prelude::{Bundle, Component, Handle, Transform},
    transform::TransformBundle,
};

use crate::{
    consts::{self, ITEM_LAYER},
    metadata::{ItemMeta, ItemSpawnMeta},
};

#[derive(Component)]
pub struct Item;

/// Represents an item, that is either on the map (waiting to be picked up), or carried.
/// If an item is on the map, it has a TransformBundle; if it's carried, it instead has a Player.
#[derive(Bundle)]
pub struct ItemBundle {
    item: Item,
    item_meta_handle: Handle<ItemMeta>,
}

impl ItemBundle {
    pub fn new(item_spawn_meta: &ItemSpawnMeta) -> Self {
        Self {
            item: Item,
            item_meta_handle: item_spawn_meta.item_handle.clone(),
        }
    }

    pub fn spawn(mut commands: EntityCommands, item_spawn_meta: &ItemSpawnMeta) {
        let ground_offset = Vec3::new(0.0, consts::GROUND_Y, ITEM_LAYER);
        let transform_bundle = TransformBundle::from_transform(Transform::from_translation(
            item_spawn_meta.location + ground_offset,
        ));

        commands.insert_bundle(transform_bundle);
    }
}
