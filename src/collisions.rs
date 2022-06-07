use bevy::{
    core::Timer,
    hierarchy::DespawnRecursiveExt,
    math::Vec2,
    prelude::{Commands, EventReader, Query, Transform, With, Without},
};
use bevy_rapier2d::prelude::*;

use crate::{attack::Attack, item::Item, movement::Knockback, state::State, Enemy, Player, Stats};

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
            let (player, enemy);
            if player_query.contains(*e1) && enemy_query.contains(*e2) {
                (player, enemy) = (*e1, *e2);
            } else {
                (player, enemy) = (*e2, *e1);
            }

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
            let (attack, enemy);
            if attack_query.contains(*e1) && enemy_query.contains(*e2) {
                (attack, enemy) = (*e1, *e2);
            } else {
                (attack, enemy) = (*e2, *e1);
            }

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
            let (item, enemy);
            if item_query.contains(*e1) && enemy_query.contains(*e2) {
                (item, enemy) = (*e1, *e2);
            } else {
                (item, enemy) = (*e2, *e1);
            }
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
        }
    }
}
