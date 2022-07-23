use bevy::{
    core::Timer,
    hierarchy::DespawnRecursiveExt,
    math::Vec2,
    prelude::{Commands, EventReader, Query, Transform, With, Without},
};
use bevy_rapier2d::prelude::*;

use crate::{
    attack::Attack, attack::Weapon, movement::Knockback, state::State, Enemy, Player, Stats,
};

#[derive(Copy, Clone)]
pub struct BodyLayers;

impl BodyLayers {
    // Each successive layer represents a different bit in the 32-bit u32 type.
    //
    // The layer is represented by 1 shifted 0 places to the left:          0b0001.
    // The second layer is represented by 1 shifted one place to the left:  0b0010.
    // And so on for the rest of the layers.
    pub const ENEMY: u32 = 1 << 0;
    pub const PLAYER: u32 = 1 << 1;
    pub const PLAYER_ATTACK: u32 = 1 << 2;
    pub const ITEM: u32 = 1 << 3;
    pub const ENEMY_ATTACK: u32 = 1 << 4;
    // u32::MAX is a u32 with all of it's bits set to 1, so this will contain all of the layers.
    pub const ALL: u32 = u32::MAX;
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

pub fn enemy_attack_player_collision(
    mut commands: Commands,
    mut events: EventReader<CollisionEvent>,
    mut player_query: Query<(&mut State, &mut Stats, &Transform), (With<Player>, Without<Enemy>)>,
    attack_query: Query<(&Attack, &Transform)>,
) {
    for event in events.iter() {
        if let CollisionEvent::Started(e1, e2, _flags) = event {
            let (attack, enemy);
            if attack_query.contains(*e1) && player_query.contains(*e2) {
                (attack, enemy) = (*e1, *e2);
            } else {
                (attack, enemy) = (*e2, *e1);
            }
            if let Ok((mut e_state, mut e_stats, e_transform)) = player_query.get_mut(enemy) {
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
    mut enemy_query: Query<(&mut State, &mut Stats, &Transform), (With<Enemy>, Without<Weapon>)>,
    item_query: Query<(&Attack, &Transform), (With<Weapon>, Without<Enemy>)>,
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
