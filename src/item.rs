use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_mod_js_scripting::{ActiveScripts, JsScript};
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::{
    animation::{AnimatedSpriteSheetBundle, Animation, Facing},
    attack::{Attack, AttackFrames, Breakable, BrokeEvent},
    collision::{BodyLayers, PhysicsBundle},
    consts,
    fighter::AvailableAttacks,
    lifetime::{Lifetime, LifetimeExpired},
    metadata::{AttackMeta, ItemKind, ItemMeta, ItemSpawnMeta},
    movement::{AngularVelocity, Force, LinearVelocity},
};

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(drop_system)
            .add_system(explodable_system)
            .add_event::<ScriptItemThrowEvent>()
            .add_event::<ScriptItemGrabEvent>();
    }
}

#[derive(Reflect, Clone)]
pub struct ScriptItemThrowEvent {
    pub fighter: Entity,
    pub script_handle: Handle<JsScript>,
}

#[derive(Reflect, Clone)]
pub struct ScriptItemGrabEvent {
    pub fighter: Entity,
    pub script_handle: Handle<JsScript>,
}

#[derive(Component)]
pub struct Item;

#[derive(Bundle)]
pub struct ItemBundle {
    item: Item,
    item_meta_handle: Handle<ItemMeta>,
    name: Name,
}

impl ItemBundle {
    pub fn new(item_spawn_meta: &ItemSpawnMeta) -> Self {
        Self {
            item: Item,
            item_meta_handle: item_spawn_meta.item_handle.clone(),
            // TODO: Actually include the item's name at some point
            name: Name::new("Map Item"),
        }
    }

    pub fn spawn(
        mut commands: EntityCommands,
        item_spawn_meta: &ItemSpawnMeta,
        items_assets: &mut ResMut<Assets<ItemMeta>>,
        active_scripts: &mut ActiveScripts,
    ) {
        let ground_offset = Vec3::new(0.0, consts::GROUND_Y, consts::ITEM_LAYER);
        let transform_bundle = TransformBundle::from_transform(Transform::from_translation(
            item_spawn_meta.location + ground_offset,
        ));

        commands.insert_bundle(transform_bundle);

        let mut item = None;
        let item_meta = items_assets
            .get_mut(&item_spawn_meta.item_handle)
            .expect("Item not loaded!");
        match &item_meta.kind {
            ItemKind::BreakableBox {
                hurtbox,
                hits,
                item_handle,
                ..
            } => {
                item = Some(item_handle.clone());

                let mut physics_bundle = PhysicsBundle::new(hurtbox, BodyLayers::BREAKABLE_ITEM);
                physics_bundle.collision_groups.filters = BodyLayers::PLAYER_ATTACK;

                commands
                    .insert_bundle(physics_bundle)
                    .insert(Breakable::new(*hits, false));
            }
            ItemKind::Script { script_handle, .. } => {
                active_scripts.insert(script_handle.clone());
            }
            _ => (),
        }

        if let Some(item) = item {
            commands.insert(Drop {
                item: items_assets.get(&item).expect("Item not loaded!").clone(),
            });
        }
    }
}

#[derive(Bundle)]
pub struct Projectile {
    #[bundle]
    sprite_bundle: SpriteBundle,
    velocity: LinearVelocity,
    angular_velocity: AngularVelocity,
    force: Force,
    collider: Collider,
    sensor: Sensor,
    events: ActiveEvents,
    collision_types: ActiveCollisionTypes,
    collision_groups: CollisionGroups,
    attack: Attack,
    lifetime: Lifetime,
    breakable: Breakable,
}

impl Projectile {
    pub fn from_thrown_item(translation: Vec3, item_meta: &ItemMeta, facing: &Facing) -> Self {
        let direction_mul = if facing.is_left() {
            Vec2::new(-1.0, 1.0)
        } else {
            Vec2::ONE
        };

        Self {
            sprite_bundle: SpriteBundle {
                texture: item_meta.image.image_handle.clone(),
                transform: Transform::from_xyz(translation.x, translation.y, consts::PROJECTILE_Z),
                ..default()
            },
            attack: Attack {
                damage: match item_meta.kind {
                    crate::metadata::ItemKind::Throwable { damage } => damage,
                    crate::metadata::ItemKind::BreakableBox { damage, .. } => damage,
                    crate::metadata::ItemKind::MeleeWeapon { .. }
                    | crate::metadata::ItemKind::ProjectileWeapon { .. } => {
                        panic!("Cannot throw weapon")
                    }
                    crate::metadata::ItemKind::Script { .. } => {
                        panic!("Cannot throw scripted items as projectiles")
                    }
                    crate::metadata::ItemKind::Bomb { .. } => {
                        panic!("Bomb is a animated projectile")
                    }
                },
                velocity: Vec2::new(consts::ITEM_ATTACK_VELOCITY, 0.0) * direction_mul,
                hitstun_duration: consts::HITSTUN_DURATION,
                hitbox_meta: None,
            },
            velocity: LinearVelocity(consts::THROW_ITEM_SPEED * direction_mul),
            // Gravity
            force: Force(Vec2::new(0.0, -consts::THROW_ITEM_GRAVITY)),
            angular_velocity: AngularVelocity(consts::THROW_ITEM_ROTATION_SPEED * direction_mul.x),
            collider: Collider::cuboid(consts::ITEM_WIDTH / 2., consts::ITEM_HEIGHT / 2.),
            sensor: Sensor,
            events: ActiveEvents::COLLISION_EVENTS,
            collision_types: ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC,
            //TODO: define collision layer based on the fighter shooting projectile, load for asset
            //files of fighter which "team" they are on
            collision_groups: CollisionGroups::new(
                BodyLayers::PLAYER_ATTACK,
                BodyLayers::ENEMY | BodyLayers::BREAKABLE_ITEM,
            ),
            lifetime: Lifetime(Timer::from_seconds(consts::THROW_ITEM_LIFETIME, false)),
            breakable: Breakable::new(0, false),
        }
    }
}

/// A component that with Breakable, drops a item when broke.
#[derive(Component, Clone)]
pub struct Drop {
    /// Item data
    pub item: ItemMeta,
}

fn drop_system(
    mut items_assets: ResMut<Assets<ItemMeta>>,
    mut commands: Commands,
    mut broke_event: EventReader<BrokeEvent>,
    mut lifetime_event: EventReader<LifetimeExpired>,
    mut active_scripts: ResMut<ActiveScripts>,
) {
    let mut drops = vec![];
    for event in lifetime_event.iter() {
        if let Some(drop) = event.drop.clone() {
            drops.push((drop, event.transform.expect("Needs transform to drop!")));
        }
    }
    for event in broke_event.iter() {
        if let Some(drop) = event.drop.clone() {
            drops.push((drop, event.transform.expect("Needs transform to drop!")));
        }
    }

    for (drop, transform) in drops {
        let ground_offset = Vec3::new(0.0, consts::GROUND_Y, consts::ITEM_LAYER);

        let item_spawn_meta = ItemSpawnMeta {
            location: transform.translation - ground_offset,
            item: String::new(),
            item_handle: items_assets.add(drop.item.clone()),
        };
        let item_commands = commands.spawn_bundle(ItemBundle::new(&item_spawn_meta));
        ItemBundle::spawn(
            item_commands,
            &item_spawn_meta,
            &mut items_assets,
            &mut active_scripts,
        );
    }
}

/// A component that with Breakable, explodes.
#[derive(Component, Clone)]
pub struct Explodable {
    pub attack: AttackMeta,
    pub timer: Timer,
    pub fusing: bool,
    pub animated_sprite: AnimatedSpriteSheetBundle,
    pub explosion_frames: AttackFrames,
}

fn explodable_system(
    mut commands: Commands,
    mut broke_event: EventReader<BrokeEvent>,
    mut explodables: Query<(
        &mut Explodable,
        &mut Force,
        &mut LinearVelocity,
        &mut AngularVelocity,
        &mut Transform,
        &mut Animation,
        Entity,
    )>,
    time: Res<Time>,
) {
    let mut explosions = Vec::new();

    for event in broke_event.iter() {
        if let Some((transform, explodable)) = event.transform.zip(event.explodable.clone()) {
            explosions.push((transform, explodable))
        }
    }

    for (
        mut explodable,
        mut force,
        mut velocity,
        mut ang_vel,
        mut transform,
        mut animation,
        entity,
    ) in &mut explodables
    {
        explodable.timer.tick(time.delta());

        if !explodable.fusing && explodable.timer.finished() {
            // Stop bomb and start fusing
            force.0 = Vec2::ZERO;
            velocity.0 = Vec2::ZERO;
            ang_vel.0 = 0.;
            transform.rotation.z = 0.;

            animation.play("bomb_fuse", false);
            explodable.fusing = true;
        } else if animation.is_finished() && explodable.fusing {
            explosions.push((*transform, explodable.clone()));
            commands.entity(entity).despawn_recursive();
        }
    }

    for (transform, explodable) in explosions {
        // Spawn explosion
        let mut animated_sprite = explodable.animated_sprite.clone();
        animated_sprite.sprite_sheet.transform = transform;
        animated_sprite.sprite_sheet.transform.rotation.z = 0.;
        animated_sprite.animation.play("explosion", false);

        let attack = explodable.attack.clone();
        let seconds = animated_sprite
            .animation
            .animations
            .get("explosion")
            .expect("No explosion animation");
        let seconds = (seconds.frames.end - seconds.frames.start) as f32
            * animated_sprite.animation.timer.duration().as_secs_f32();

        let attack_ent = commands
            .spawn()
            .insert(Sensor)
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC)
            .insert(CollisionGroups::new(
                BodyLayers::ENEMY_ATTACK,
                BodyLayers::PLAYER,
            ))
            .insert(Attack {
                damage: attack.damage,
                velocity: attack.velocity.unwrap_or(Vec2::ZERO),
                hitstun_duration: attack.hitstun_duration,
            })
            .insert(explodable.explosion_frames)
            .insert(transform)
            .id();

        commands
            .spawn_bundle(animated_sprite)
            .insert(Lifetime(Timer::from_seconds(seconds, false)))
            .insert(AvailableAttacks {
                attacks: vec![explodable.attack],
            })
            .push_children(&[attack_ent]);
    }
}

#[derive(Bundle)]
pub struct AnimatedProjectile {
    #[bundle]
    sprite_bundle: AnimatedSpriteSheetBundle,
    velocity: LinearVelocity,
    angular_velocity: AngularVelocity,
    force: Force,
    collider: Collider,
    sensor: Sensor,
    events: ActiveEvents,
    collision_types: ActiveCollisionTypes,
    collision_groups: CollisionGroups,
    attack: Attack,
    breakable: Breakable,
}

impl AnimatedProjectile {
    pub fn new(damage: i32, facing: &Facing, animated_sprite: AnimatedSpriteSheetBundle) -> Self {
        let direction_mul = if facing.is_left() {
            Vec2::new(-1.0, 1.0)
        } else {
            Vec2::ONE
        };
        let mut rng = rand::thread_rng();

        Self {
            sprite_bundle: animated_sprite,
            attack: Attack {
                damage,
                velocity: Vec2::new(consts::ITEM_ATTACK_VELOCITY, 0.0) * direction_mul,
                hitstun_duration: consts::HITSTUN_DURATION,
            },
            velocity: LinearVelocity(
                consts::THROW_ITEM_SPEED * direction_mul * rng.gen_range(0.8..1.2),
            ),
            // Gravity
            force: Force(Vec2::new(0.0, -consts::THROW_ITEM_GRAVITY)),
            angular_velocity: AngularVelocity(
                consts::THROW_ITEM_ROTATION_SPEED * direction_mul.x * rng.gen_range(0.8..1.2),
            ),
            collider: Collider::cuboid(consts::ITEM_WIDTH / 2., consts::ITEM_HEIGHT / 2.),
            sensor: Sensor,
            events: ActiveEvents::COLLISION_EVENTS,
            collision_types: ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC,
            //TODO: define collision layer based on the fighter shooting projectile, load for asset
            //files of fighter which "team" they are on
            collision_groups: CollisionGroups::new(BodyLayers::ENEMY_ATTACK, BodyLayers::PLAYER),
            breakable: Breakable::new(0, false),
        }
    }
}
