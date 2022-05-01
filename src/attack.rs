use bevy::{
    input::Input,
    math::Vec3,
    prelude::{
        App, Commands, Component, Deref, DerefMut, KeyCode, Plugin, Query, Res, Transform, With,
    },
    transform::TransformBundle,
};
use heron::{CollisionLayers, CollisionShape, RigidBody, Velocity};

use crate::{
    animation::Facing,
    collisions::BodyLayers,
    consts::{ATTACK_HEIGHT, ATTACK_LAYER, ATTACK_WIDTH},
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
    pub damage: Damage,
}

#[derive(Component, Deref, DerefMut)]
pub struct Damage(pub i32);

fn player_attack(
    query: Query<(&Transform, &Facing), With<Player>>,
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    keyboard: Res<Input<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Return) {
        let (transform, facing) = query.single();
        let mut dir = Vec3::X;

        if facing.is_left() {
            dir = -dir;
        }

        commands
            .spawn_bundle(TransformBundle::from_transform(Transform::from_xyz(
                transform.translation.x,
                transform.translation.y,
                ATTACK_LAYER,
            )))
            .insert(RigidBody::KinematicVelocityBased)
            .insert(CollisionShape::Cuboid {
                half_extends: Vec3::new(ATTACK_WIDTH / 2., ATTACK_HEIGHT / 2., 0.),
                border_radius: None,
            })
            .insert(facing.clone())
            .insert(CollisionLayers::new(
                BodyLayers::PlayerAttack,
                BodyLayers::Enemy,
            ))
            .insert(Velocity::from_linear(dir * 300.))
            .insert(Attack { damage: Damage(10) });
    }
}
