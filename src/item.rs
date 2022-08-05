use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::{
    consts,
    metadata::{ItemMeta, ItemSpawnMeta},
};

#[derive(Component)]
pub struct Item;

/// Represents an item, that is either on the map.
#[derive(Bundle)]
pub struct ItemBundle {
    item: Item,
    item_meta_handle: Handle<ItemMeta>,
    name: Name,
}

impl ItemBundle {
    pub fn new(item_spawn_meta: &ItemSpawnMeta) -> Self {
        Self {
            item: Item,
            item_meta_handle: item_spawn_meta.item_handle.clone(),
            // TODO: Actually include the item's name somehow
            name: Name::new("Map Item"),
        }
    }

    pub fn spawn(mut commands: EntityCommands, item_spawn_meta: &ItemSpawnMeta) {
        let ground_offset = Vec3::new(0.0, consts::GROUND_Y, consts::ITEM_LAYER);
        let transform_bundle = TransformBundle::from_transform(Transform::from_translation(
            item_spawn_meta.location + ground_offset,
        ));

        commands.insert_bundle(transform_bundle);
    }
}
