use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::{
    animation::Facing,
    attack::Attack,
    collision::BodyLayers,
    consts,
    lifetime::Lifetime,
    metadata::{ItemMeta, ItemSpawnMeta},
    movement::{AngularVelocity, Force, LinearVelocity},
};

#[derive(Component)]
pub struct Item;

/// Represents an item, that is either on the map.
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

    pub fn spawn(mut commands: EntityCommands, item_spawn_meta: &ItemSpawnMeta) {
        let ground_offset = Vec3::new(0.0, consts::GROUND_Y, consts::ITEM_LAYER);
        let transform_bundle = TransformBundle::from_transform(Transform::from_translation(
            item_spawn_meta.location + ground_offset,
        ));

        commands.insert_bundle(transform_bundle);
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
}

impl Projectile {
    pub fn from_thrown_item(translation: Vec3, item: &ItemMeta, facing: &Facing) -> Self {
        let direction_mul = if facing.is_left() {
            Vec2::new(-1.0, 1.0)
        } else {
            Vec2::ONE
        };

        Self {
            sprite_bundle: SpriteBundle {
                texture: item.image.image_handle.clone(),
                transform: Transform::from_xyz(translation.x, translation.y, consts::PROJECTILE_Z),
                ..default()
            },
            attack: Attack {
                damage: item.damage,
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
        }
    }
}
