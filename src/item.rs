use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::{
    animation::Facing,
    attack::{Attack, Breakable},
    collision::{BodyLayers, PhysicsBundle},
    consts,
    lifetime::Lifetime,
    metadata::{ItemKind, ItemMeta, ItemSpawnMeta},
    movement::{AngularVelocity, Force, LinearVelocity},
};

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

        let item_meta = items_assets
            .get_mut(&item_spawn_meta.item_handle)
            .expect("Item not loaded!");
        if let ItemKind::BreakableBox {
            hurtbox,
            hits,
            item,
            ..
        } = &item_meta.kind
        {
            let mut physics_bundle = PhysicsBundle::new(hurtbox, BodyLayers::BREAKABLE_ITEM);
            physics_bundle.collision_groups.filters = BodyLayers::PLAYER_ATTACK;

            commands
                .insert_bundle(physics_bundle)
                .insert(Breakable::new(*hits))
                .insert(Drop {
                    item: *item.clone(),
                    drop: false,
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
            collision_groups: CollisionGroups::new(BodyLayers::PLAYER_ATTACK, BodyLayers::ENEMY),
            lifetime: Lifetime(Timer::from_seconds(consts::THROW_ITEM_LIFETIME, false)),
            breakable: Breakable::new(0),
        }
    }
}

/// A component that with Breakable, drops a item when broke.
#[derive(Component)]
pub struct Drop {
    /// Item data
    pub item: ItemMeta,
    /// Set true to drop
    pub drop: bool,
}
