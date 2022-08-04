//! Enemy fighter AI

use bevy::prelude::*;
use rand::{prelude::SliceRandom, Rng};

use crate::{
    animation::Facing,
    commands::CustomCommands,
    consts,
    enemy::{Enemy, TripPointX},
    fighter_state::{
        Attacking, Idling, Moving, StateTransition, StateTransitionIntents, TransitionCmds,
    },
    player::Player,
    Stats,
};

/// A place that an enemy fighter is going to move to
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct EnemyTarget {
    pub position: Vec2,
}

// For enemys without current target, pick a new spot near the player as target
///
/// This is added to the [`crate::fighter_state::FighterStateCollectSystems`] to collect figher
/// actions for enemies.
pub fn set_target_near_player(
    mut commands: Commands,
    mut enemies_query: Query<
        (Entity, &mut TripPointX),
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
        for (e_entity, mut e_trip_point_x) in enemies_query.iter_mut() {
            if let Some(p_transform) = p_transforms.choose(&mut rng) {
                if max_player_x > e_trip_point_x.0 {
                    e_trip_point_x.0 = f32::MIN;

                    let x_offset = rng.gen_range(-100.0..100.);
                    let y_offset = rng.gen_range(-100.0..100.);
                    commands.entity(e_entity).insert(EnemyTarget {
                        position: Vec2::new(
                            p_transform.translation.x + x_offset,
                            (p_transform.translation.y + y_offset)
                                .clamp(consts::MIN_Y, consts::MAX_Y),
                        ),
                    });
                }
            }
        }
    }
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
        ),
        // All enemies that are either moving or idling
        (With<Enemy>, Or<(With<Idling>, With<Moving>)>),
    >,
    mut transition_commands: CustomCommands<TransitionCmds>,
    time: Res<Time>,
) {
    let mut commands = transition_commands.commands();

    for (entity, transform, stats, target, mut facing, mut intents) in &mut query {
        let position = transform.translation.truncate();
        let velocity =
            (target.position - position).normalize() * stats.movement_speed * time.delta_seconds();

        if velocity.x < 0.0 {
            *facing = Facing::Left;
        } else {
            *facing = Facing::Right;
        }

        // If we're close to our target
        if position.distance(target.position) <= 100. {
            // Remove the target
            commands.entity(entity).remove::<EnemyTarget>();

            // And attack!
            intents.push_back(StateTransition::new(
                Attacking::default(),
                Attacking::PRIORITY,
                false,
            ));

        // If we aren't near our target yet
        } else {
            // Move towards our target
            intents.push_back(StateTransition::new(
                Moving { velocity },
                Moving::PRIORITY,
                false,
            ));
        }
    }
}
