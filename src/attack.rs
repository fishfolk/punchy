use bevy::{hierarchy::DespawnRecursiveExt, math::Vec2, prelude::*, reflect::Reflect};
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

use serde::Deserialize;

use crate::{
    animation::Animation,
    damage::{DamageEvent, Damageable, Health},
    metadata::{FighterMeta, ItemMeta, ItemSpawnMeta},
    GameState,
    item::ItemBundle
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

#[derive(Component)]
pub struct Drop {
    pub item: Handle<ItemMeta>,
    pub location: Vec3,
}

/// A component that depawns an entity after collision.
#[derive(Component, Clone, Copy, Default, Reflect)]
pub struct Breakable {
    /// The number of collisions allowed before the entity is breakable.
    pub hit_tolerance: i32,
    /// The number of collisions occured.
    pub hit_count: i32,
}

impl Breakable {
    pub fn new(hits: i32) -> Self {
        Self {
            hit_tolerance: hits,
            hit_count: 0,
        }
    }
}

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
                        fighter_data.attack.hitbox.size.x / 2.,
                        fighter_data.attack.hitbox.size.y / 2.,
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

fn breakable_system(
    mut events: EventReader<CollisionEvent>,
    mut despawn_query: Query<(&mut Breakable, Option<&Drop>)>,
    mut commands: Commands,
    items_assets: Res<Assets<ItemMeta>>,
) {
    for ev in events.iter() {
        if let CollisionEvent::Started(e1, e2, _flags) = ev {
            for e in [e1, e2].iter() {
                if let Ok(mut breakable) = despawn_query.get_mut(**e) {
                    let drop = breakable.1;
                    let breakable = &mut breakable.0;

                    if breakable.hit_count < breakable.hit_tolerance {
                        breakable.hit_count += 1;
                    } else {
                        commands.entity(**e).despawn_recursive();

                        if let Some(drop) = drop {
                            let item_spawn_meta = ItemSpawnMeta {
                                location: drop.location,
                                item: String::new(),
                                item_handle: drop.item.clone(),
                            };
                            let item_bundle = ItemBundle::new(&item_spawn_meta); //ItemBundle {item: Item, name: Name::new("Map Item"), item_meta_handle: drop.item.clone()};
                            let item_commands = commands.spawn_bundle(item_bundle);
                            ItemBundle::spawn(item_commands, &item_spawn_meta, &items_assets)
                        }
                    }
                }
            }
        }
    }
}
