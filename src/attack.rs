use bevy::{
    hierarchy::DespawnRecursiveExt,
    prelude::{App, Commands, Component, Entity, Parent, Plugin, Query, With, Without},
};
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    animation::Animation,
    consts::{ATTACK_HEIGHT, ATTACK_WIDTH},
    GameState,
};

pub struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                // .with_system(player_projectile_attack)
                // .with_system(player_throw)
                // .with_system(player_flop)
                .with_system(activate_hitbox)
                .with_system(deactivate_hitbox)
                // .with_system(projectile_cleanup)
                // .with_system(projectile_tick)
                .into(),
        );
        // .add_system(
        //     enemy_attack.run_in_state(GameState::InGame), // .after("move_to_target"),
        // );
    }
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component)]
pub struct Attack {
    pub damage: i32,
}

#[derive(Component)]
pub struct AttackFrames {
    pub startup: usize,
    pub active: usize,
    pub recovery: usize,
}

fn activate_hitbox(
    attack_query: Query<(Entity, &AttackFrames, &Parent), Without<Collider>>,
    fighter_query: Query<&Animation>,
    mut commands: Commands,
) {
    for (entity, attack_frames, parent) in attack_query.iter() {
        if let Ok(animation) = fighter_query.get(**parent) {
            if animation.current_frame >= attack_frames.startup
                && animation.current_frame <= attack_frames.active
            {
                //TODO: insert Collider based on size and transform offset in attack asset
                commands
                    .entity(entity)
                    .insert(Collider::cuboid(ATTACK_WIDTH * 0.8, ATTACK_HEIGHT * 0.8));
            }
        }
    }
}

fn deactivate_hitbox(
    query: Query<(Entity, &AttackFrames, &Parent), (With<Attack>, With<Collider>)>,
    fighter_query: Query<&Animation>,
    mut commands: Commands,
) {
    for (entity, attack_frames, parent) in query.iter() {
        if let Ok(animation) = fighter_query.get(**parent) {
            if animation.current_frame >= attack_frames.recovery {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
