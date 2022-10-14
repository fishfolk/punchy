use bevy::prelude::*;
use rand::prelude::SliceRandom;
use serde::Deserialize;

use crate::attack::Hurtbox;
use crate::consts::{self, FOOT_PADDING};
use crate::metadata::ItemMeta;
use crate::{
    animation::{AnimatedSpriteSheetBundle, Animation, Facing, SyncAnimation, SyncFacing},
    camera::YSort,
    collision::{BodyLayers, PhysicsBundle},
    damage::{Damageable, Health},
    enemy::Enemy,
    fighter_state::{Idling, StateTransitionIntents},
    metadata::FighterMeta,
    movement::LinearVelocity,
    player::Player,
};

pub struct FighterPlugin;

impl Plugin for FighterPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PostUpdate, attachment_system);
    }
}

/// Bundle added to a fighter stub, in order to activate it.
#[derive(Bundle)]
pub struct ActiveFighterBundle {
    pub name: Name,
    #[bundle]
    pub animated_spritesheet_bundle: AnimatedSpriteSheetBundle,
    // #[bundle]
    // pub physics_bundle: PhysicsBundle,
    pub stats: Stats,
    pub ysort: YSort,
    pub health: Health,
    pub damageable: Damageable,
    pub inventory: Inventory,
    pub state_transition_intents: StateTransitionIntents,
    /// Fighters start off idling, but this component may be removed when the fighter state changes.
    pub idling: Idling,
    pub velocity: LinearVelocity,
}

#[derive(Component, Deserialize, Clone, Debug, Reflect)]
#[reflect(Component)]
#[serde(deny_unknown_fields)]
pub struct Stats {
    pub max_health: i32,
    pub movement_speed: f32,
}

/// The player inventory.
///
/// A player may be holding one item
#[derive(Component, Clone, Debug, Default, Deref, DerefMut)]
pub struct Inventory(pub Option<ItemMeta>);

impl Default for Stats {
    fn default() -> Self {
        Stats {
            max_health: 100,
            movement_speed: 17000.,
        }
    }
}

/// Turns a fighter stub data (loaded from the metadata) into a fully active fighter.
impl ActiveFighterBundle {
    pub fn activate_fighter_stub(
        commands: &mut Commands,
        fighter: &FighterMeta,
        entity: Entity,
        transform: &Transform,
        player: Option<&Player>,
        enemy: Option<&Enemy>,
    ) {
        let body_layers = if player.is_some() {
            BodyLayers::PLAYER
        } else if enemy.is_some() {
            BodyLayers::ENEMY
        } else {
            unreachable!();
        };

        let active_fighter_bundle = ActiveFighterBundle {
            name: Name::new(fighter.name.clone()),
            animated_spritesheet_bundle: AnimatedSpriteSheetBundle {
                sprite_sheet: SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        anchor: bevy::sprite::Anchor::Custom(Vec2::new(
                            0.,
                            //calculate anchor to align with feet
                            0.5 * FOOT_PADDING / fighter.center_y - 0.5,
                        )),
                        ..default()
                    },
                    texture_atlas: fighter
                        .spritesheet
                        .atlas_handle
                        .choose(&mut rand::thread_rng())
                        .unwrap()
                        .clone(),
                    transform: *transform,
                    ..Default::default()
                },
                animation: Animation::new(
                    fighter.spritesheet.animation_fps,
                    fighter.spritesheet.animations.clone(),
                ),
            },
            stats: fighter.stats.clone(),
            health: Health(fighter.stats.max_health),
            inventory: default(),
            damageable: default(),
            // physics_bundle: PhysicsBundle::new(&fighter.hurtbox, body_layers),
            idling: Idling,
            state_transition_intents: default(),
            // ysort: YSort(fighter.spritesheet.tile_size.y as f32 / 2.),
            ysort: YSort(consts::FIGHTERS_Z),
            velocity: default(),
        };
        let hurtbox = commands
            .spawn_bundle(PhysicsBundle::new(&fighter.hurtbox, body_layers))
            .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
                0.0,
                fighter.collision_offset,
                0.0,
            )))
            .insert(Hurtbox)
            .id();

        let animated_spritesheet_bundle = active_fighter_bundle.animated_spritesheet_bundle.clone();

        commands
            .entity(entity)
            .insert_bundle(active_fighter_bundle)
            .push_children(&[hurtbox]);

        if let Some(attachment) = &fighter.attachment {
            //Clone fighter spritesheet
            let mut attachment_spritesheet = animated_spritesheet_bundle;

            //Change what's needed
            attachment_spritesheet.sprite_sheet.texture_atlas = attachment
                .atlas_handle
                .choose(&mut rand::thread_rng())
                .unwrap()
                .clone();
            attachment_spritesheet.animation =
                Animation::new(attachment.animation_fps, attachment.animations.clone());
            attachment_spritesheet.sprite_sheet.transform = Transform::from_xyz(0., fighter.spritesheet.tile_size.y as f32 * 0.3, 0.1);
            attachment_spritesheet.sprite_sheet.sprite.anchor =  bevy::sprite::Anchor::Center;

            let attachment_ent = commands
                .spawn_bundle(attachment_spritesheet)
                .insert(Facing::default())
                .insert(SyncFacing)
                .insert(SyncAnimation)
                .id();
            commands.entity(entity).add_child(attachment_ent);
        }
    }
}

/// Standard way to attach things to fighters
/// Needs Facing component
#[derive(Component)]
pub struct Attached {
    ///Change position based on facing
    pub position_face: bool,
}

pub fn attachment_system(mut query: Query<(&Attached, &mut Transform, &Facing)>) {
    for (attached, mut transform, facing) in &mut query {
        if attached.position_face {
            transform.translation.x = if facing.is_left() {
                -transform.translation.x.abs()
            } else {
                transform.translation.x.abs()
            };
        }
    }
}
