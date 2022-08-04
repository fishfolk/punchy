use bevy::prelude::*;
use bevy_rapier2d::prelude::CollisionGroups;
use rand::prelude::SliceRandom;

use crate::{
    animation::Animation,
    collisions::BodyLayers,
    enemy::Enemy,
    fighter_state::{Idling, StateTransitionIntents},
    metadata::FighterMeta,
    movement::LinearVelocity,
    player::Player,
    AnimatedSpriteSheetBundle, CharacterBundle, PhysicsBundle,
};

/// Bundle added to a fighter stub, in order to activate it.
#[derive(Bundle)]
pub struct ActiveFighterBundle {
    name: Name,
    #[bundle]
    animated_spritesheet_bundle: AnimatedSpriteSheetBundle,
    #[bundle]
    character_bundle: CharacterBundle,
    #[bundle]
    physics_bundle: PhysicsBundle,
    state_transition_intents: StateTransitionIntents,
    /// Fighters start off idling, but this component may be removed when the fighter state changes.
    idling: Idling,
    velocity: LinearVelocity,
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
            character_bundle: CharacterBundle {
                stats: fighter.stats.clone(),
                ..default()
            },
            physics_bundle: PhysicsBundle {
                collision_groups: CollisionGroups::new(body_layers, BodyLayers::ALL),
                ..default()
            },
            idling: Idling,
            state_transition_intents: default(),
            velocity: default(),
        };

        commands.entity(entity).insert_bundle(active_fighter_bundle);
    }
}
