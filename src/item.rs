use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::{
    animation::Facing,
    attack::{Attack, Breakable, BrokeEvent},
    collision::{BodyLayers, PhysicsBundle},
    consts,
    lifetime::{Lifetime, LifetimeExpired},
    metadata::{ItemKind, ItemMeta, ItemSpawnMeta},
    movement::{AngularVelocity, Force, LinearVelocity},
};

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(drop_system);
    }
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
        if let ItemKind::BreakableBox {
            hurtbox,
            hits,
            item_handle,
            ..
        } = &item_meta.kind
        {
            item = Some(item_handle.clone());

            let mut physics_bundle = PhysicsBundle::new(hurtbox, BodyLayers::BREAKABLE_ITEM);
            physics_bundle.collision_groups.filters = BodyLayers::PLAYER_ATTACK;

            commands
                .insert_bundle(physics_bundle)
                .insert(Breakable::new(*hits, false));
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
                    crate::metadata::ItemKind::Health { .. } => panic!("Cannot throw health item"),
                    crate::metadata::ItemKind::BreakableBox { damage, .. } => damage,
                    crate::metadata::ItemKind::MeleeWeapon { .. }
                    | crate::metadata::ItemKind::ProjectileWeapon { .. } => {
                        panic!("Cannot throw weapon")
                    }
                },
                velocity: Vec2::new(consts::ATTACK_VELOCITY, 0.0) * direction_mul,
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
        ItemBundle::spawn(item_commands, &item_spawn_meta, &mut items_assets);
    }
}
