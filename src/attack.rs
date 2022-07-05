use std::time::Duration;

use bevy::{
    core::{Time, Timer},
    hierarchy::{BuildChildren, DespawnRecursiveExt},
    input::Input,
    math::Vec2,
    prelude::{
        App, Commands, Component, Entity, EventReader, KeyCode, Plugin, Query, Res, Transform, With,
    },
    transform::TransformBundle,
};
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    animation::Facing,
    collisions::BodyLayers,
    consts::{ATTACK_HEIGHT, ATTACK_LAYER, ATTACK_WIDTH},
    movement::{MoveInDirection, Target},
    state::State,
    ArrivedEvent, Enemy, GameState, Player,
};

pub struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(player_attack.run_in_state(GameState::InGame));
    }
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component)]
pub struct Attack {
    pub damage: i32,
}

#[derive(Component)]
pub struct AttackTimer(pub Timer);

fn player_attack(
    query: Query<(&Transform, &Facing), With<Player>>,
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Return) {
        for (transform, facing) in query.iter() {
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
                    BodyLayers::PLAYER_ATTACK,
                    BodyLayers::ENEMY,
                ))
                .insert(facing.clone())
                .insert(MoveInDirection(dir * 300.)) //TODO: Put the velocity in a const
                // .insert(Velocity::from_linear(dir * 300.))
                .insert(Attack { damage: 10 })
                .insert(AttackTimer(Timer::new(Duration::from_secs(1), false)));
        }
    }
}

pub fn enemy_attack(
    mut query: Query<&mut State, (With<Enemy>, With<Target>)>,
    mut event_reader: EventReader<ArrivedEvent>,
    mut commands: Commands,
) {
    for event in event_reader.iter() {
        if let Ok(mut state) = query.get_mut(event.0) {
            if *state != State::Attacking {
                state.set(State::Attacking);
                let attack_entity = commands
                    .spawn_bundle(TransformBundle::default())
                    .insert(Collider::cuboid(ATTACK_WIDTH * 1.2, ATTACK_HEIGHT * 1.2))
                    .insert(Sensor(true))
                    .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC)
                    .insert(CollisionGroups::new(
                        BodyLayers::ENEMY_ATTACK,
                        BodyLayers::PLAYER,
                    ))
                    .insert(Attack { damage: 10 })
                    .insert(AttackTimer(Timer::new(Duration::from_secs_f32(0.5), false)))
                    .id();
                commands.entity(event.0).push_children(&[attack_entity]);
            }
        }
    }
}

pub fn attack_cleanup(query: Query<(Entity, &AttackTimer), With<Attack>>, mut commands: Commands) {
    for (entity, timer) in query.iter() {
        if timer.0.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn attack_tick(mut query: Query<&mut AttackTimer, With<Attack>>, time: Res<Time>) {
    for mut timer in query.iter_mut() {
        timer.0.tick(time.delta());
    }
}
