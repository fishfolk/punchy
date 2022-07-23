use bevy::{
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
