use std::collections::HashSet;

use bevy::{
    ecs::system::EntityCommands,
    math::Vec3,
    prelude::{Bundle, Commands, Component, Entity, Handle, Query, Transform, With},
    transform::TransformBundle,
};
use leafwing_input_manager::prelude::ActionState;

use crate::{
    consts::{self, ITEM_LAYER, PICK_ITEM_RADIUS},
    input::PlayerAction,
    metadata::{ItemMeta, ItemSpawnMeta},
    player::Player,
    state::State,
};

#[derive(Component)]
pub struct Item;

#[derive(Component)]
pub struct CarriedBy(pub Entity);

/// Represents an item, that is either on the map (waiting to be picked up), or carried.
/// If an item is on the map, it has a TransformBundle; if it's carried, it instead has a
/// CarriedBy.
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

pub fn pick_items(
    mut commands: Commands,
    player_query: Query<(Entity, &Transform, &State, &ActionState<PlayerAction>), With<Player>>,
    items_query: Query<(Entity, &Transform), With<Item>>,
) {
    // We need to track the picked items, otherwise, in theory, two players could pick the same item.
    let mut picked_item_ids = HashSet::new();

    for (player_id, player_transform, player_state, input) in player_query.iter() {
        if *player_state != State::Idle && *player_state != State::Running {
            continue;
        }

        if input.just_pressed(PlayerAction::Throw) {
            // If several items are at pick distance, an arbitrary one is picked.
            for (item_id, item_transform) in items_query.iter() {
                if !picked_item_ids.contains(&item_id) {
                    let player_item_distance = player_transform
                        .translation
                        .truncate()
                        .distance(item_transform.translation.truncate());

                    if player_item_distance <= PICK_ITEM_RADIUS {
                        commands
                            .entity(item_id)
                            .remove_bundle::<TransformBundle>()
                            .insert(CarriedBy(player_id));
                        picked_item_ids.insert(item_id);
                        break;
                    }
                }
            }
        }
    }
}
