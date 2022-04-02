use bevy::{
    core::Time,
    input::Input,
    math::Vec2,
    prelude::{KeyCode, Query, Res, Transform},
};

use crate::{consts, state::State, Player};

pub fn player_controller(
    mut query: Query<(&mut Player, &mut Transform)>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (mut player, mut transform) = query.single_mut();

    if player.state == State::ATTACKING {
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
    dir = dir.normalize_or_zero() * player.movement_speed * time.delta_seconds();

    //Restrict player to the ground
    let new_y = transform.translation.y + dir.y + consts::GROUND_OFFSET;

    if new_y >= consts::MAX_Y || new_y <= consts::MIN_Y {
        dir.y = 0.;
    }

    //Move the player
    transform.translation.x += dir.x;
    transform.translation.y += dir.y;

    //Set the player state and direction
    if dir.x > 0. {
        player.facing_left = false;
    } else if dir.x < 0. {
        player.facing_left = true;
    }

    if dir == Vec2::ZERO {
        player.state = State::IDLE;
    } else {
        player.state = State::RUNNING;
    }
}
