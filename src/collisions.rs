use bevy::{
    hierarchy::DespawnRecursiveExt,
    math::Vec2,
    prelude::{Commands, EventReader, Query, Transform},
    time::Timer,
};
use bevy_rapier2d::prelude::*;

use crate::{
    attack::{Attack, ProjectileLifetime},
    movement::Knockback,
    Stats,
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
    pub const ENEMY_ATTACK: u32 = 1 << 3;
    pub const ITEM: u32 = 1 << 4;
    // u32::MAX is a u32 with all of it's bits set to 1, so this will contain all of the layers.
    pub const ALL: u32 = u32::MAX;
}

pub fn attack_fighter_collision(
    mut commands: Commands,
    mut events: EventReader<CollisionEvent>,
    mut fighter_query: Query<(&mut Stats, &Transform)>,
    attack_query: Query<(&Attack, &Transform, Option<&ProjectileLifetime>)>,
) {
    // for event in events.iter() {
    //     if let CollisionEvent::Started(e1, e2, _flags) = event {
    //         let (attack_entity, fighter_entity) = if attack_query.contains(*e1) {
    //             // In this case, it's guaranteed that e1 is found (as projectile), but e2 and the
    //             // entities in the else case, may potentially not be found.
    //             (*e1, *e2)
    //         } else {
    //             (*e2, *e1)
    //         };

    //         if let Ok((mut f_state, mut f_stats, f_transform)) =
    //             fighter_query.get_mut(fighter_entity)
    //         {
    //             if let Ok((a_attack, a_transform, maybe_projectile)) =
    //                 attack_query.get(attack_entity)
    //             {
    //                 f_stats.health -= a_attack.damage;

    //                 let force = 150.; //TODO: set this to a constant
    //                 let mut direction = Vec2::new(0., 0.);

    //                 if a_transform.translation.x < f_transform.translation.x {
    //                     f_state.set(State::KnockedLeft);
    //                     direction.x = force;
    //                 } else {
    //                     f_state.set(State::KnockedRight);
    //                     direction.x = -force;
    //                 }

    //                 commands.entity(fighter_entity).insert(Knockback {
    //                     direction,
    //                     duration: Timer::from_seconds(0.15, false),
    //                 });

    //                 if maybe_projectile.is_some() {
    //                     commands.entity(attack_entity).despawn_recursive();
    //                 }
    //             }
    //         }
    //     }
    // }
}
