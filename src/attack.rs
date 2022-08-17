use bevy::{hierarchy::DespawnRecursiveExt, math::Vec2, prelude::*, reflect::Reflect};
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

use serde::Deserialize;

use crate::{
    animation::Animation,
    damage::{DamageEvent, Damageable, Health},
    metadata::FighterMeta,
    GameState,
};

pub struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register reflect types
            .register_type::<Attack>()
            // Add systems
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(activate_hitbox)
                    .with_system(deactivate_hitbox)
                    .with_system(despawn_collision)
                    .into(),
            )
            // Attack damage is run in PostUpdate to make sure it runs after rapier generates collision events
            .add_system_to_stage(CoreStage::PostUpdate, attack_damage_system);
    }
}

/// A component representing an attack that can do damage to [`Damageable`]s with [`Health`].
#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct Attack {
    pub damage: i32,
    /// The direction and speed that the attack is hitting something in.
    pub velocity: Vec2,
}

/// A component that depawns an entity after collision with tolerance to x collisions.
#[derive(Component, Deref, DerefMut, Clone, Copy, Default, Reflect)]
pub struct DespawnOnCollision(pub u32);

/// A component identifying the attacks active collision frames.
///
/// Must be added to an entity that is a child of an entity with an [`Animation`] and an [`Attack`]
/// and will be used to spawn a collider for that attack during the `active` frames.
/// Each field is an index refering to an animation frame
#[derive(Component, Debug, Clone, Copy, Deserialize)]
pub struct AttackFrames {
    pub startup: usize,
    pub active: usize,
    pub recovery: usize,
}

fn activate_hitbox(
    attack_query: Query<(Entity, &AttackFrames, &Parent), Without<Collider>>,
    fighter_query: Query<(&Animation, &Handle<FighterMeta>)>,
    mut commands: Commands,
    fighter_assets: Res<Assets<FighterMeta>>,
) {
    for (entity, attack_frames, parent) in attack_query.iter() {
        if let Ok((animation, fighter_meta)) = fighter_query.get(**parent) {
            if animation.current_frame >= attack_frames.startup
                && animation.current_frame <= attack_frames.active
            {
                if let Some(fighter_data) = fighter_assets.get(fighter_meta) {
                    commands.entity(entity).insert(Collider::cuboid(
                        fighter_data.attack.hitbox.x,
                        fighter_data.attack.hitbox.y,
                    ));
                }
            }
        }
    }
}

/// Deactivate collisions for entities with [`AttackFrames`]
fn deactivate_hitbox(
    query: Query<(Entity, &AttackFrames, &Parent), (With<Attack>, With<Collider>)>,
    animated_query: Query<&Animation>,
    mut commands: Commands,
) {
    for (entity, attack_frames, parent) in query.iter() {
        if let Ok(animation) = animated_query.get(**parent) {
            if animation.current_frame >= attack_frames.recovery {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

/// Depletes the health of damageables that have collided with attacks
fn attack_damage_system(
    mut events: EventReader<CollisionEvent>,
    mut damageables: Query<(&mut Health, &Damageable)>,
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
            let (mut health, damageable) = damageables.get_mut(damageable_entity).unwrap();

            if **damageable {
                **health -= attack.damage;

                event_writer.send(DamageEvent {
                    damageing_entity: attack_entity,
                    damage_velocity: attack.velocity,
                    damage: attack.damage,
                    damaged_entity: damageable_entity,
                })
            }
        }
    }
}

fn despawn_collision(
    mut events: EventReader<CollisionEvent>,
    mut despawn_query: Query<&mut DespawnOnCollision>,
    mut commands: Commands,
) {
    for ev in events.iter() {
        if let CollisionEvent::Started(e1, e2, _flags) = ev {
            for e in [e1, e2].iter() {
                if let Ok(mut despawn) = despawn_query.get_mut(**e) {
                    if **despawn > 0 {
                        **despawn -= 1;
                    } else {
                        commands.entity(**e).despawn_recursive();
                    }
                }
            }
        }
    }
}
