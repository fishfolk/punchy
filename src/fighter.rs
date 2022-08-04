use bevy::prelude::*;
use bevy_rapier2d::prelude::CollisionGroups;
use rand::prelude::SliceRandom;
use serde::Deserialize;

use crate::{
    animation::{AnimatedSpriteSheetBundle, Animation},
    collisions::{BodyLayers, PhysicsBundle},
    damage::{Damageable, Health},
    enemy::Enemy,
    fighter_state::{Idling, StateTransitionIntents},
    metadata::FighterMeta,
    movement::LinearVelocity,
    player::Player,
    y_sort::YSort,
};

/// Bundle added to a fighter stub, in order to activate it.
#[derive(Bundle)]
pub struct ActiveFighterBundle {
    pub name: Name,
    #[bundle]
    pub animated_spritesheet_bundle: AnimatedSpriteSheetBundle,
    #[bundle]
    pub physics_bundle: PhysicsBundle,
    pub stats: Stats,
    pub ysort: YSort,
    pub health: Health,
    pub damageable: Damageable,
    pub state_transition_intents: StateTransitionIntents,
    /// Fighters start off idling, but this component may be removed when the fighter state changes.
    pub idling: Idling,
    pub velocity: LinearVelocity,
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Stats {
    pub max_health: i32,
    pub damage: i32,
    pub movement_speed: f32,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            max_health: 100,
            damage: 35,
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
                    sprite: TextureAtlasSprite::new(0),
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
            damageable: default(),
            physics_bundle: PhysicsBundle {
                collision_groups: CollisionGroups::new(body_layers, BodyLayers::ALL),
                ..default()
            },
            idling: Idling,
            state_transition_intents: default(),
            ysort: default(),
            velocity: default(),
        };

        commands.entity(entity).insert_bundle(active_fighter_bundle);
    }
}
