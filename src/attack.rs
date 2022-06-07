use bevy::{
    input::Input,
    math::{Vec2, Vec3},
    prelude::{App, Commands, Component, KeyCode, Plugin, Query, Res, Transform, With},
    transform::TransformBundle,
};
use bevy_rapier2d::prelude::*;

use crate::{
    animation::Facing,
    collisions::BodyLayers,
    consts::{ATTACK_HEIGHT, ATTACK_LAYER, ATTACK_WIDTH},
    movement::MoveInDirection,
    Player,
};

pub struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(player_attack);
    }
}

#[derive(Component)]
pub struct Attack {
    pub damage: i32,
}

fn player_attack(
    query: Query<(&Transform, &Facing), With<Player>>,
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    keyboard: Res<Input<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Return) {
        let (transform, facing) = query.single();
        let mut dir = Vec2::X;

        if facing.is_left() {
            dir = -dir;
        }

        commands
            .spawn_bundle(TransformBundle::from_transform(Transform::from_xyz(
                transform.translation.x,
                transform.translation.y,
                ATTACK_LAYER,
            )))
            .insert(Collider::cuboid(ATTACK_WIDTH / 2., ATTACK_HEIGHT / 2.))
            .insert(Sensor(true))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC)
            .insert(CollisionGroups::new(
                BodyLayers::PlayerAttack as u32,
                BodyLayers::Enemy as u32,
            ))
            .insert(facing.clone())
            .insert(MoveInDirection(dir * 300.)) //TODO: Put the velocity in a const
            // .insert(Velocity::from_linear(dir * 300.))
            .insert(Attack { damage: 10 });
    }
}
