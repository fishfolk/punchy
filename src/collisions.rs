use bevy::{
    core::Timer,
    hierarchy::DespawnRecursiveExt,
    math::Vec2,
    prelude::{Commands, EventReader, Query, Transform, With, Without},
};
use bevy_rapier2d::prelude::*;
// use heron::{CollisionEvent, PhysicsLayer};

use crate::{attack::Attack, item::Item, movement::Knockback, state::State, Enemy, Player, Stats};

// #[derive(PhysicsLayer)]
pub enum BodyLayers {
    Enemy = 0b0001,
    Player = 0b0010,
    PlayerAttack = 0b0100,
    Item = 0b1000,
}

pub fn player_enemy_collision(
    mut commands: Commands,
    mut events: EventReader<CollisionEvent>,
    mut enemy_query: Query<(&mut State, &mut Stats, &Transform), (With<Enemy>, Without<Player>)>,
    player_query: Query<(&State, &Stats, &Transform), (With<Player>, Without<Enemy>)>,
) {
    for event in events.iter() {
        if let CollisionEvent::Started(e1, e2, _flags) = event {
            //     }
            // }

            // events
            //     .iter()
            //     .filter(|e| matches!(e, CollisionEvent::Started { .. }))
            //     .for_each(|e| {
            //         // let entity1 = e.
            // let (e1, e2) = (e.0, e.1);
            // let (e1, e2) = e.rigid_body_entities();
            // let (l1, l2) = e.collision_layers();

            // let (player, enemy);
            // if l1.contains_group(BodyLayers::Player) && l2.contains_group(BodyLayers::Enemy) {
            //     player = e1;
            //     enemy = e2;
            // } else if l2.contains_group(BodyLayers::Player) && l1.contains_group(BodyLayers::Enemy)
            // {
            //     player = e2;
            //     enemy = e1;
            // } else {
            //     return;
            // }
            let (player, enemy) = (*e1, *e2);

            if let Ok((mut e_state, mut e_stats, e_transform)) = enemy_query.get_mut(enemy) {
                if let Ok((p_state, p_stats, p_transform)) = player_query.get(player) {
                    if *p_state == State::Attacking {
                        e_stats.health -= p_stats.damage;
                        println! {"e_stats.health {:?}", e_stats.health};

                        let force = 150.; //TODO: set this to a constant
                        let mut direction = Vec2::new(0., 0.);

                        if p_transform.translation.x < e_transform.translation.x {
                            e_state.set(State::KnockedLeft);
                            direction.x = force;
                        } else {
                            e_state.set(State::KnockedRight);
                            direction.x = -force;
                        }

                        commands.entity(enemy).insert(Knockback {
                            direction,
                            duration: Timer::from_seconds(0.15, false),
                        });
                    }
                }
            }
            // });
        }
    }
}

pub fn player_attack_enemy_collision(
    mut commands: Commands,
    mut events: EventReader<CollisionEvent>,
    mut enemy_query: Query<(&mut State, &mut Stats, &Transform), (With<Enemy>, Without<Player>)>,
    attack_query: Query<(&Attack, &Transform)>,
) {
    for event in events.iter() {
        if let CollisionEvent::Started(e1, e2, _flags) = event {
            // events.iter().filter(|e| e.is_started()).for_each(|e| {
            //     let (e1, e2) = e.rigid_body_entities();
            //     let (l1, l2) = e.collision_layers();

            //     let (attack, enemy);
            //     if l1.contains_group(BodyLayers::PlayerAttack) && l2.contains_group(BodyLayers::Enemy) {
            //         attack = e1;
            //         enemy = e2;
            //     } else if l2.contains_group(BodyLayers::PlayerAttack)
            //         && l1.contains_group(BodyLayers::Enemy)
            //     {
            //         attack = e2;
            //         enemy = e1;
            //     } else {
            //         return;
            //     }

            let (attack, enemy) = (*e1, *e2);

            if let Ok((mut e_state, mut e_stats, e_transform)) = enemy_query.get_mut(enemy) {
                if let Ok((a_attack, a_transform)) = attack_query.get(attack) {
                    e_stats.health -= a_attack.damage;

                    let force = 150.; //TODO: set this to a constant
                    let mut direction = Vec2::new(0., 0.);

                    if a_transform.translation.x < e_transform.translation.x {
                        e_state.set(State::KnockedLeft);
                        direction.x = force;
                    } else {
                        e_state.set(State::KnockedRight);
                        direction.x = -force;
                    }

                    commands.entity(enemy).insert(Knockback {
                        direction,
                        duration: Timer::from_seconds(0.15, false),
                    });

                    commands.entity(attack).despawn_recursive();
                }
            }
            // });
        }
    }
}

pub fn item_attacks_enemy_collision(
    mut commands: Commands,
    mut events: EventReader<CollisionEvent>,
    mut enemy_query: Query<(&mut State, &mut Stats, &Transform), (With<Enemy>, Without<Item>)>,
    item_query: Query<(&Attack, &Transform), (With<Item>, Without<Enemy>)>,
) {
    for event in events.iter() {
        if let CollisionEvent::Started(e1, e2, _flags) = event {
            // events.iter().filter(|e| e.is_started()).for_each(|e| {
            //     let (e1, e2) = e.rigid_body_entities();
            //     let (l1, l2) = e.collision_layers();

            //     let (item, enemy);
            //     if l1.contains_group(BodyLayers::Item) && l2.contains_group(BodyLayers::Enemy) {
            //         item = e1;
            //         enemy = e2;
            //     } else if l2.contains_group(BodyLayers::Item) && l1.contains_group(BodyLayers::Enemy) {
            //         item = e2;
            //         enemy = e1;
            //     } else {
            //         return;
            //     }
            let (item, enemy) = (*e1, *e2);

            if let Ok((mut e_state, mut e_stats, e_transform)) = enemy_query.get_mut(enemy) {
                if let Ok((a_attack, a_transform)) = item_query.get(item) {
                    e_stats.health -= a_attack.damage;

                    let force = 150.; //TODO: set this to a constant
                    let mut direction = Vec2::new(0., 0.);

                    if a_transform.translation.x < e_transform.translation.x {
                        e_state.set(State::KnockedLeft);
                        direction.x = force;
                    } else {
                        e_state.set(State::KnockedRight);
                        direction.x = -force;
                    }

                    commands.entity(enemy).insert(Knockback {
                        direction,
                        duration: Timer::from_seconds(0.15, false),
                    });

                    commands.entity(item).despawn_recursive();
                };
            };
            // });
        }
    }
}
