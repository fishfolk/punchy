use bevy::{
    math::{Vec2, Vec3},
    prelude::{default, AssetServer, Commands, Component, EventReader, Res, Transform},
    sprite::SpriteBundle,
};
// use heron::{CollisionLayers, CollisionShape, RigidBody};
use bevy_rapier2d::prelude::*;

use crate::{
    animation::Facing,
    attack::Attack,
    collisions::BodyLayers,
    consts::{self, ITEM_HEIGHT, ITEM_LAYER, ITEM_WIDTH},
    movement::{MoveInArc, Rotate},
};

#[derive(Component)]
pub struct Item;

#[derive(Component)]
struct Pickable;

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

        commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load("bottled_seaweed11x31.png"),
                transform: Transform::from_xyz(ev.position.x, ev.position.y, ITEM_LAYER),
                ..default()
            })
            .insert(Item)
            .insert(Pickable)
            // .insert(RigidBody::Sensor)
            .insert(Sensor(true))
            // .insert(CollisionShape::Cuboid {
            //     half_extends: Vec3::new(ITEM_WIDTH / 2., ITEM_HEIGHT / 2., 0.),
            //     border_radius: None,
            // })
            .insert(Collider::cuboid(ITEM_WIDTH / 2., ITEM_HEIGHT / 2.))
            .insert(CollisionGroups::new(
                BodyLayers::Item as u32,
                BodyLayers::Enemy as u32,
            ))
            // .insert(CollisionLayers::new(BodyLayers::Item, BodyLayers::Enemy))
            .insert(Attack {
                damage: consts::THROW_ITEM_DAMAGE,
            })
            .insert(Rotate {
                speed: consts::THROW_ITEM_ROTATION_SPEED,
                to_right: !ev.facing.is_left(),
            })
            .insert(MoveInArc {
                //TODO: Set in consts
                radius: Vec2::new(
                    50.,
                    consts::PLAYER_HEIGHT + consts::THROW_ITEM_Y_OFFSET + consts::ITEM_HEIGHT,
                ),
                speed: consts::THROW_ITEM_SPEED,
                angle: angles.0,
                end_angle: angles.1,
                inverse_direction: ev.facing.is_left(),
                origin: ev.position,
            });
    }
}
