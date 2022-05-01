use bevy::{
    core::{Time, Timer},
    input::Input,
    math::Vec2,
    prelude::{Commands, Component, Entity, KeyCode, Query, Res, Transform, With},
};

use crate::{animation::Facing, consts, state::State, Player, Stats};

#[derive(Component)]
pub struct Knockback {
    pub direction: Vec2,
    pub duration: Timer,
}

pub fn knockback_system(
    mut query: Query<(Entity, &mut Transform, &mut Knockback)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut transform, mut knockback) in query.iter_mut() {
        if knockback.duration.finished() {
            commands.entity(entity).remove::<Knockback>();
        } else {
            transform.translation.x += knockback.direction.x * time.delta_seconds();
            transform.translation.y += knockback.direction.y * time.delta_seconds();
            knockback.duration.tick(time.delta());
        }
    }
}

pub fn player_controller(
    mut query: Query<(&mut State, &Stats, &mut Transform, Option<&mut Facing>), With<Player>>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (mut state, stats, mut transform, facing_option) = query.single_mut();

    if *state == State::Attacking {
        return;
    }

    let mut dir = Vec2::ZERO;

    if keyboard.pressed(KeyCode::A) {
        dir -= Vec2::X;
    }

    if keyboard.pressed(KeyCode::D) {
        dir += Vec2::X;
    }

    if keyboard.pressed(KeyCode::W) {
        dir += Vec2::Y;
    }

    if keyboard.pressed(KeyCode::S) {
        dir -= Vec2::Y;
    }

    //Normalize direction
    dir = dir.normalize_or_zero() * stats.movement_speed * time.delta_seconds();

    //Restrict player to the ground
    let new_y = transform.translation.y + dir.y + consts::GROUND_OFFSET;

    if new_y >= consts::MAX_Y || new_y <= consts::MIN_Y {
        dir.y = 0.;
    }

    //Move the player
    transform.translation.x += dir.x;
    transform.translation.y += dir.y;

    //Set the player state and direction
    if let Some(mut facing) = facing_option {
        if dir.x < 0. {
            facing.set(Facing::Left);
        } else if dir.x > 0. {
            facing.set(Facing::Right);
        }
    }

    if dir == Vec2::ZERO {
        *state = State::Idle;
    } else {
        *state = State::Running;
    }
}
