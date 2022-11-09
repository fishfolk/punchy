//! Enemy fighter AI

use bevy::prelude::*;
use rand::Rng;

use crate::{
    animation::Facing,
    consts::{self, ENEMY_MAX_ATTACK_DISTANCE, ENEMY_MIN_ATTACK_DISTANCE, ENEMY_TARGET_MAX_OFFSET},
    enemy::{Boss, Enemy, TripPointX},
    fighter_state::{
        GroundSlam, Idling, Moving, Punching, StateTransition, StateTransitionIntents,
    },
    player::Player,
    Stats,
};

//maybe implement as plugin

/// A place that an enemy fighter is going to move to, in an attempt to attack a player.
///
/// The attack distance is for randomization purposes, and it's the distance that triggers the
/// attack. More precisely, it's the max distance - if the enemy finds itself at a smaller distance,
/// it will attack.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct EnemyTarget {
    pub position: Vec2,
    pub attack_distance: f32,
}

// For enemys without current target, pick a new spot near the player as target
///
/// This is added to the [`crate::fighter_state::FighterStateCollectSystems`] to collect figher
/// actions for enemies.
pub fn set_target_near_player(
    mut commands: Commands,
    mut enemies_query: Query<
        (Entity, &mut TripPointX, &Transform),
        (With<Enemy>, With<Idling>, Without<EnemyTarget>),
    >,
    player_query: Query<&Transform, With<Player>>,
) {
    let mut rng = rand::thread_rng();
    let p_transforms = player_query.iter().collect::<Vec<_>>();
    let max_player_x = p_transforms
        .iter()
        .map(|transform| transform.translation.x)
        .max_by(f32::total_cmp);

    if let Some(max_player_x) = max_player_x {
        for (e_entity, mut e_trip_point_x, e_transform) in enemies_query.iter_mut() {
            if let Some(p_transform) = choose_player(&p_transforms, e_transform) {
                if max_player_x > e_trip_point_x.0 {
                    e_trip_point_x.0 = f32::MIN;

                    let x_offset = rng.gen_range(-ENEMY_TARGET_MAX_OFFSET..ENEMY_TARGET_MAX_OFFSET);
                    let y_offset = rng.gen_range(-ENEMY_TARGET_MAX_OFFSET..ENEMY_TARGET_MAX_OFFSET);

                    let attack_distance =
                        rng.gen_range(ENEMY_MIN_ATTACK_DISTANCE..ENEMY_MAX_ATTACK_DISTANCE);

                    commands.entity(e_entity).insert(EnemyTarget {
                        position: Vec2::new(
                            p_transform.translation.x + x_offset,
                            (p_transform.translation.y + y_offset)
                                .clamp(consts::MIN_Y, consts::MAX_Y),
                        ),
                        attack_distance,
                    });
                }
            }
        }
    }
}

/// Chooses which player is closer
pub fn choose_player(p_transforms: &Vec<&Transform>, e_transform: &Transform) -> Option<Transform> {
    if !p_transforms.is_empty() {
        let mut closer = (p_transforms[0], dist(p_transforms[0], e_transform));

        for transform in p_transforms.iter().skip(1) {
            let dist = dist(transform, e_transform);

            if dist < closer.1 {
                closer.0 = transform;
                closer.1 = dist;
            }
        }

        Some(*closer.0)
    } else {
        None
    }
}

pub fn dist(transform1: &Transform, transform2: &Transform) -> f32 {
    ((transform1.translation.x - transform2.translation.x).powi(2)
        + (transform1.translation.y - transform2.translation.y).powi(2))
    .sqrt()
}

/// Controls enemy AI fighters
///
/// This is added to the [`crate::fighter_state::FighterStateCollectSystems`] to collect figher
/// actions for enemies.
pub fn emit_enemy_intents(
    mut query: Query<
        (
            Entity,
            &Transform,
            &Stats,
            &EnemyTarget,
            &mut Facing,
            &mut StateTransitionIntents,
            Option<&Boss>,
        ),
        // All enemies that are either moving or idling
        (With<Enemy>, Or<(With<Idling>, With<Moving>)>),
    >,
    mut commands: Commands,
) {
    for (entity, transform, stats, target, mut facing, mut intents, maybe_boss) in &mut query {
        let position = transform.translation.truncate();
        let velocity = (target.position - position).normalize() * stats.movement_speed;

        // If we're close to our target
        if position.distance(target.position) <= target.attack_distance {
            // Note that the target includes an offset, so this can still not point to the
            // player.

            // Remove the target
            commands.entity(entity).remove::<EnemyTarget>();

            // Face the target position
            *facing = if target.position.x > position.x {
                Facing::Right
            } else {
                Facing::Left
            };

            // make them attack with their first available attack??
            // And attack!
            if maybe_boss.is_some() {
                intents.push_back(StateTransition::new(
                    GroundSlam::default(),
                    GroundSlam::PRIORITY,
                    false,
                ))
            } else {
                intents.push_back(StateTransition::new(
                    Punching::default(),
                    Punching::PRIORITY,
                    false,
                ));
            }
        // If we aren't near our target yet
        } else {
            // Face the cirection we're moving
            *facing = if velocity.x < 0.0 {
                Facing::Left
            } else {
                Facing::Right
            };

            // Move towards our target
            intents.push_back(StateTransition::new(
                Moving { velocity },
                Moving::PRIORITY,
                false,
            ));
        }
    }
}
