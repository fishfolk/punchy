use bevy::{
    hierarchy::DespawnRecursiveExt,
    prelude::{
        App, Commands, Component, CoreStage, Entity, EventReader, EventWriter, Parent, Plugin,
        Query, With, Without,
    },
};
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    animation::Animation,
    consts::{ATTACK_HEIGHT, ATTACK_WIDTH},
    damage::{DamageEvent, Damageable, Health},
    GameState,
};

pub struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add systems
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(activate_hitbox)
                    .with_system(deactivate_hitbox)
                    .into(),
            )
            // Attack damage is run in PostUpdate to make sure it runs after rapier generates collision events
            .add_system_to_stage(CoreStage::PostUpdate, attack_damage_system);
    }
}

/// A component representing an attack that can do damage to [`Damageable`]s with [`Health`].
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component)]
pub struct Attack {
    pub damage: i32,
}

/// A component identifying the attacks active collision frames.
///
/// Must be added to an entity that is a child of an entity with an [`Animation`] and an [`Attack`]
/// and will be used to spawn a collider for that attack during the `active` frames.
#[derive(Component)]
pub struct AttackFrames {
    pub startup: usize,
    pub active: usize,
    pub recovery: usize,
}

/// Activate collisions for entities with [`AttackFrames`]
fn activate_hitbox(
    attack_query: Query<(Entity, &AttackFrames, &Parent), Without<Collider>>,
    animated_query: Query<&Animation>,
    mut commands: Commands,
) {
    for (entity, attack_frames, parent) in attack_query.iter() {
        if let Ok(animation) = animated_query.get(**parent) {
            if animation.current_frame >= attack_frames.startup
                && animation.current_frame <= attack_frames.active
            {
                //TODO: insert Collider based on size and transform offset in attack asset
                commands
                    .entity(entity)
                    .insert(Collider::cuboid(ATTACK_WIDTH * 0.8, ATTACK_HEIGHT * 0.8));
            }
        }
    }
}

/// Deactivate collisions for entities with [`AttackFrames`]
fn deactivate_hitbox(
    query: Query<(Entity, &AttackFrames, &Parent), (With<Attack>, With<Collider>)>,
    fighter_query: Query<&Animation>,
    mut commands: Commands,
) {
    for (entity, attack_frames, parent) in query.iter() {
        if let Ok(animation) = fighter_query.get(**parent) {
            if animation.current_frame >= attack_frames.recovery {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

/// Depletes the health of damageables that have collided with attacks
fn attack_damage_system(
    mut events: EventReader<CollisionEvent>,
    mut damageables: Query<&mut Health, With<Damageable>>,
    attacks: Query<&Attack>,
    mut event_writer: EventWriter<DamageEvent>,
) {
    for event in events.iter() {
        if let CollisionEvent::Started(e1, e2, _flags) = event {
            let (attack_entity, damageable_entity) =
                if attacks.contains(*e1) && damageables.contains(*e2) {
                    (*e1, *e2)
                } else if attacks.contains(*e2) && damageables.contains(*e1) {
                    (*e2, *e1)
                } else {
                    continue;
                };

            let attack = attacks.get(attack_entity).unwrap();
            let mut health = damageables.get_mut(damageable_entity).unwrap();
            **health -= attack.damage;

            event_writer.send(DamageEvent {
                attack_entity,
                damaged_entity: damageable_entity,
                damage: attack.damage,
            })
        }
    }
}
