use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    animation::Facing,
    attack::Attack,
    collision::BodyLayers,
    consts,
    lifetime::Lifetime,
    metadata::ItemMeta,
    movement::{AngularVelocity, Force, LinearVelocity},
};

#[derive(Component, Clone, Debug)]
pub struct Item {
    pub name: Name,
    pub meta_handle: Handle<ItemMeta>,
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
    item: Item,
}

impl Projectile {
    pub fn from_thrown_item(
        translation: Vec3,
        item: Item,
        item_meta: &ItemMeta,
        facing: &Facing,
    ) -> Self {
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
            item,
        }
    }
}
