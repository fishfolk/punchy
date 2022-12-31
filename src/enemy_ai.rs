//! Enemy fighter AI

use bevy::prelude::*;
use rand::Rng;

use crate::{
    animation::Facing,
    consts::{self, ENEMY_MAX_ATTACK_DISTANCE, ENEMY_MIN_ATTACK_DISTANCE, ENEMY_TARGET_MAX_OFFSET},
    enemy::{Boss, Enemy, TripPointX},
    fighter::AvailableAttacks,
    fighter_state::{
        BossBombThrow, Idling, Moving, ProjectileAttacking, Punching, StateTransition,
        StateTransitionIntents,
    },
    metadata::{ItemKind, ItemMeta},
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
pub struct WalkTarget {
    pub position: Vec2,
    pub attack_distance: f32,
    pub player_pos: Vec2,
}

// For enemys without current target, pick a new spot near the player as target
///
/// This is added to the [`crate::fighter_state::FighterStateCollectSystems`] to collect figher
/// actions for enemies.
pub fn set_move_target_near_player(
    mut commands: Commands,
    mut enemies_query: Query<
        (Entity, &mut TripPointX, &Transform, &AvailableAttacks),
        (With<Enemy>, With<Idling>, Without<WalkTarget>),
    >,
    player_query: Query<&Transform, With<Player>>,
    items_assets: Res<Assets<ItemMeta>>,
) {
    let mut rng = rand::thread_rng();
    let p_transforms = player_query.iter().collect::<Vec<_>>();
    let max_player_x = p_transforms
        .iter()
        .map(|transform| transform.translation.x)
        .max_by(f32::total_cmp);

    if let Some(max_player_x) = max_player_x {
        for (e_entity, mut e_trip_point_x, e_transform, available_attacks) in
            enemies_query.iter_mut()
        {
            if let Some(p_transform) = choose_player(&p_transforms, e_transform) {
                if max_player_x > e_trip_point_x.0 {
                    e_trip_point_x.0 = f32::MIN;

                    let mut x_offset =
                        rng.gen_range(-ENEMY_TARGET_MAX_OFFSET..ENEMY_TARGET_MAX_OFFSET);
                    let mut y_offset =
                        rng.gen_range(-ENEMY_TARGET_MAX_OFFSET..ENEMY_TARGET_MAX_OFFSET);

                    let cur_attack = available_attacks.current_attack();
                    if cur_attack.name.as_str() == "projectile" {
                        let item = items_assets
                            .get(&cur_attack.item_handle)
                            .expect("No item found.");

                        if let ItemKind::Throwable {
                            lifetime,
                            throw_velocity,
                            gravity,
                            ..
                        } = item.kind
                        {
                            //Change target offset to aim on player
                            x_offset += throw_velocity.x
                                * (lifetime * 0.65)
                                * if p_transform.translation.x > e_transform.translation.x {
                                    -1.
                                } else {
                                    1.
                                };

                            y_offset += (throw_velocity.y + gravity) * (lifetime * 0.65);
                        }
                    }

                    let attack_distance =
                        rng.gen_range(ENEMY_MIN_ATTACK_DISTANCE..ENEMY_MAX_ATTACK_DISTANCE);

                    commands.entity(e_entity).insert(WalkTarget {
                        position: Vec2::new(
                            p_transform.translation.x + x_offset,
                            (p_transform.translation.y + y_offset)
                                .clamp(consts::MIN_Y, consts::MAX_Y),
                        ),
                        attack_distance,
                        player_pos: p_transform.translation.truncate(),
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
            &WalkTarget,
            &mut Facing,
            &mut StateTransitionIntents,
            Option<&Boss>,
            &AvailableAttacks,
        ),
        // All enemies that are either moving or idling
        (With<Enemy>, Or<(With<Idling>, With<Moving>)>),
    >,
    mut commands: Commands,
) {
    for (
        entity,
        transform,
        stats,
        target,
        mut facing,
        mut intents,
        maybe_boss,
        available_attacks,
    ) in &mut query
    {
        let position = transform.translation.truncate();
        let velocity = (target.position - position).normalize() * stats.movement_speed;

        // If we're close to our target
        if position.distance(target.position) <= target.attack_distance {
            // Note that the target includes an offset, so this can still not point to the
            // player.

            // Remove the target
            commands.entity(entity).remove::<WalkTarget>();

            // Face the target position
            *facing = if target.position.x > position.x {
                Facing::Right
            } else {
                Facing::Left
            };

            // And attack!
            if maybe_boss.is_some() {
                // TODO Add some proper ai for the bomb throw
                intents.push_back(StateTransition::new(
                    BossBombThrow::default(),
                    BossBombThrow::PRIORITY,
                    false,
                ))
            } else {
                match available_attacks.current_attack().name.as_str() {
                    "punch" => intents.push_back(StateTransition::new(
                        Punching::default(),
                        Punching::PRIORITY,
                        false,
                    )),
                    "projectile" => {
                        // Face the player
                        *facing = if target.player_pos.x > position.x {
                            Facing::Right
                        } else {
                            Facing::Left
                        };

                        intents.push_back(StateTransition::new(
                            ProjectileAttacking::default(),
                            ProjectileAttacking::PRIORITY,
                            false,
                        ));
                    }
                    _ => {}
                }
            }
        // If we aren't near our target yet
        } else {
            // Face the direction we're moving
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
