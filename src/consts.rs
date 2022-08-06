use bevy::math::Vec2;

pub const PLAYER_SPRITE_WIDTH: f32 = 96.;
pub const PLAYER_HITBOX_HEIGHT: f32 = 50.;

/// Absolute value.
pub const ENEMY_TARGET_MAX_OFFSET: f32 = 40.;

pub const ENEMY_MIN_ATTACK_DISTANCE: f32 = 5.;
pub const ENEMY_MAX_ATTACK_DISTANCE: f32 = 100.;

// Distance from the player, after which the player movement boundary is moved forward.
//
pub const LEFT_BOUNDARY_MAX_DISTANCE: f32 = 380.;

pub const GROUND_Y: f32 = -120.;
pub const GROUND_HEIGHT: f32 = 150.;
pub const GROUND_OFFSET: f32 = 0.;

pub const CAMERA_SPEED: f32 = 0.8;

pub const MAX_Y: f32 = (GROUND_HEIGHT / 2.) + GROUND_Y;
pub const MIN_Y: f32 = -(GROUND_HEIGHT / 2.) + GROUND_Y;

pub const ATTACK_WIDTH: f32 = 16.;
pub const ATTACK_HEIGHT: f32 = 16.;
pub const ATTACK_VELOCITY: f32 = 250.0;

pub const ITEM_LAYER: f32 = 100.;
pub const ITEM_WIDTH: f32 = 30.;
pub const ITEM_HEIGHT: f32 = 10.;

pub const PROJECTILE_Z: f32 = 101.;
pub const THROW_ITEM_OFFSET: Vec2 = Vec2::from_array([5.0, 30.0]);
pub const THROW_ITEM_SPEED: Vec2 = Vec2::from_array([200.0, 300.0]);
pub const THROW_ITEM_LIFETIME: f32 = 0.64;
pub const THROW_ITEM_ROTATION_SPEED: f32 = -20.;
pub const THROW_ITEM_GRAVITY: f32 = 1200.0;

pub const PICK_ITEM_RADIUS: f32 = 24.;
