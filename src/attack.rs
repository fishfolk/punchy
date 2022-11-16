use bevy::{
    hierarchy::DespawnRecursiveExt,
    math::Vec2,
    prelude::*,
    reflect::{FromReflect, Reflect},
};
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

use serde::Deserialize;

use crate::{
    animation::Animation,
    damage::{DamageEvent, Damageable, Health},
    fighter::AvailableAttacks,
    item::Drop,
    metadata::ColliderMeta,
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
                    .with_system(breakable_system)
                    .into(),
            )
            // Attack damage is run in PostUpdate to make sure it runs after rapier generates collision events
            .add_system_to_stage(CoreStage::PostUpdate, attack_damage_system)
            // Event for when Breakable breaks
            .add_event::<BrokeEvent>();
    }
}

/// A component representing an attack that can do damage to [`Damageable`]s with [`Health`].
#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct Attack {
    //maybe just replace all fields with AttackMeta
    pub damage: i32,
    /// The direction and speed that the attack is hitting something in.
    pub velocity: Vec2,
    pub hitstun_duration: f32,
    /// add this for attacks that are not immediately active, used in activate_hitbox
    pub hitbox_meta: Option<ColliderMeta>,
}

#[derive(Component)]
pub struct Hurtbox;

/// A component that depawns an entity after collision.
#[derive(Component, Clone, Copy, Default, Reflect)]
pub struct Breakable {
    /// The number of collisions allowed before the entity is breakable.
    pub hit_tolerance: i32,
    /// The number of collisions occured.
    pub hit_count: i32,
    /// If it should despawn it's parent on break
    pub despawn_parent: bool,
}

impl Breakable {
    pub fn new(hits: i32, despawn_parent: bool) -> Self {
        Self {
            hit_tolerance: hits,
            hit_count: 0,
            despawn_parent,
        }
    }
}

pub struct BrokeEvent {
    pub drop: Option<Drop>,
    pub transform: Option<Transform>,
}

/// A component identifying the attacks active collision frames.
///
/// Must be added to an entity that is a child of an entity with an [`Animation`] and an [`Attack`]
/// and will be used to spawn a collider for that attack during the `active` frames.
/// Each field is an index refering to an animation frame
#[derive(Component, Debug, Clone, Copy, Deserialize, Reflect, FromReflect)]
pub struct AttackFrames {
    pub startup: usize,
    pub active: usize,
    pub recovery: usize,
}

/// Activates inactive attacks after the animation on the attack reaches the active frames by
/// adding a collider to the attack entity.
//TODO: is there a way we can move the adding of collision layers here as well?
fn activate_hitbox(
    attack_query: Query<(Entity, &Attack, &AttackFrames, &Parent), Without<Collider>>,
    parent_query: Query<&Animation, With<AvailableAttacks>>,
    mut commands: Commands,
) {
    for (entity, attack, attack_frames, parent) in attack_query.iter() {
        if let Ok(animation) = parent_query.get(**parent) {
            if animation.current_frame >= attack_frames.startup
                && animation.current_frame <= attack_frames.active
            {
                if let Some(hitbox_meta) = attack.hitbox_meta {
                    commands
                        .entity(entity)
                        .insert(Sensor)
                        .insert(ActiveEvents::COLLISION_EVENTS)
                        .insert(
                            ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC,
                        )
                        .insert(Collider::cuboid(
                            hitbox_meta.size.x / 2.,
                            hitbox_meta.size.y / 2.,
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
    hurtboxes: Query<&Parent, With<Hurtbox>>,
    mut event_writer: EventWriter<DamageEvent>,
) {
    for event in events.iter() {
        if let CollisionEvent::Started(e1, e2, _flags) = event {
            let (attack_entity, hurtbox_entity) =
                if attacks.contains(*e1) && hurtboxes.contains(*e2) {
                    (*e1, *e2)
                } else if attacks.contains(*e2) && hurtboxes.contains(*e1) {
                    (*e2, *e1)
                } else {
                    continue;
                };

            let attack = attacks.get(attack_entity).unwrap();
            if let Ok(hurtbox_parent) = hurtboxes.get(hurtbox_entity) {
                let hurtbox_parent_entity = hurtbox_parent.get();
                let (mut health, damageable) = damageables.get_mut(hurtbox_parent_entity).unwrap();

                //apply damage to target
                if **damageable {
                    **health -= attack.damage;

                    event_writer.send(DamageEvent {
                        damageing_entity: attack_entity,
                        damage_velocity: attack.velocity,
                        damage: attack.damage,
                        damaged_entity: hurtbox_parent_entity,
                        hitstun_duration: attack.hitstun_duration,
                    })
                }
            }
        }
    }
}

fn breakable_system(
    mut events: EventReader<CollisionEvent>,
    mut despawn_query: Query<(
        &mut Breakable,
        Option<&Drop>,
        Option<&Transform>,
        Option<&Parent>,
    )>,
    mut commands: Commands,
    mut event_writer: EventWriter<BrokeEvent>,
) {
    for ev in events.iter() {
        if let CollisionEvent::Started(e1, e2, _flags) = ev {
            for e in [e1, e2].iter() {
                if let Ok((mut breakable, drop, transform, parent)) = despawn_query.get_mut(**e) {
                    if breakable.hit_count < breakable.hit_tolerance {
                        breakable.hit_count += 1;
                    } else {
                        event_writer.send(BrokeEvent {
                            drop: drop.cloned(),
                            transform: transform.cloned(),
                        });
                        commands.entity(**e).despawn_recursive();

                        if breakable.despawn_parent {
                            if let Some(parent) = parent {
                                commands.entity(parent.get()).despawn_recursive()
                            }
                        }
                    }
                }
            }
        }
    }
}
