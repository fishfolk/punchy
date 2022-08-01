use std::collections::HashMap;

use bevy::{
    math::{Quat, Vec2},
    prelude::{
        Commands, Component, Deref, DerefMut, Entity, EventReader, EventWriter, Query, Res, ResMut,
        Transform, With,
    },
    time::{Time, Timer},
};
use leafwing_input_manager::prelude::ActionState;

use crate::{
    animation::Facing,
    consts::{self, LEFT_BOUNDARY_MAX_DISTANCE},
    input::PlayerAction,
    metadata::GameMeta,
    state::State,
    ArrivedEvent, DespawnMarker, Player, Stats,
};

/// Non-player direction, e.g. weapons.
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Deref, DerefMut)]
pub struct MoveInDirection(pub Vec2);

/// Describes the intention (of any type, not only from controls) of a player, to move. Such
/// events are processed (clamping, etc.), then applied.
/// The Vec2 must be the total displacement value, that is, calculated in the given time.
#[derive(Debug)]
pub struct PlayerMovement {
    pub player_id: Entity,
    pub movement: Vec2,
    pub set_facing_state: bool,
}

// (Moving) bondary before which, the players can't go back.
#[derive(Component)]
pub struct LeftMovementBoundary(f32);

impl Default for LeftMovementBoundary {
    fn default() -> Self {
        Self(-LEFT_BOUNDARY_MAX_DISTANCE)
    }
}

#[derive(Component)]
pub struct Knockback {
    pub direction: Vec2,
    pub duration: Timer,
}

pub fn knockback_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Knockback, Option<&Player>)>,
    time: Res<Time>,
    mut move_commands: EventWriter<PlayerMovement>,
) {
    let mut all_knockbacks = query.iter_mut().collect::<Vec<_>>();

    // Separate the finished knockbacks, and despawn them.

    let (finished_knockbacks, mut current_knockbacks): (Vec<_>, Vec<_>) = all_knockbacks
        .iter_mut()
        .partition(|(_, _, knockback, _)| knockback.duration.finished());

    for (entity, _, _, _) in &finished_knockbacks {
        commands.entity(*entity).remove::<Knockback>();
    }

    // Tick the timer for the current knockbacks.

    for (_, _, knockback, _) in current_knockbacks.iter_mut() {
        knockback.duration.tick(time.delta());
    }

    // Separate the enemy knocbacks, and apply them, unclamped.

    let (mut enemy_knockbacks, player_knockbacks): (Vec<_>, Vec<_>) = current_knockbacks
        .iter_mut()
        .partition(|(_, _, _, player)| player.is_none());

    for (_, transform, knockback, _) in enemy_knockbacks.iter_mut() {
        transform.translation.x += knockback.direction.x * time.delta_seconds();
        transform.translation.y += knockback.direction.y * time.delta_seconds();
    }

    // Extract the players movement data, and apply the knockbacks, clamped.

    for (player_id, _, knockback, _) in &player_knockbacks {
        let movement = knockback.direction * time.delta_seconds();

        move_commands.send(PlayerMovement {
            player_id: *player_id,
            movement,
            set_facing_state: false,
        })
    }
}

pub fn player_controller(
    query: Query<(Entity, &mut State, &Stats, &ActionState<PlayerAction>), With<Player>>,
    time: Res<Time>,
    mut move_commands: EventWriter<PlayerMovement>,
) {
    for (player_id, state, stats, input) in query.iter() {
        if *state == State::Idle || *state == State::Running {
            let mut movement = if input.pressed(PlayerAction::Move) {
                input.axis_pair(PlayerAction::Move).unwrap().xy()
            } else {
                Vec2::ZERO
            };

            // Apply speed
            movement = movement * stats.movement_speed * time.delta_seconds();

            //Move the player
            let event = PlayerMovement {
                player_id,
                movement,
                set_facing_state: true,
            };

            move_commands.send(event);
        }
    }
}

/// Does not apply to players.
pub fn move_direction_system(
    mut query: Query<(&mut Transform, &MoveInDirection)>,
    time: Res<Time>,
) {
    for (mut transform, dir) in &mut query.iter_mut() {
        transform.translation += dir.0.extend(0.) * time.delta_seconds();
    }
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component)]
pub struct MoveInArc {
    pub radius: Vec2,
    pub speed: f32,
    pub angle: f32,
    pub end_angle: f32,
    pub inverse_direction: bool,
    pub origin: Vec2,
}

pub fn move_in_arc_system(
    mut query: Query<(&mut Transform, &mut MoveInArc, Entity)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (mut transform, mut arc, entity) in &mut query.iter_mut() {
        if arc.inverse_direction {
            arc.angle += time.delta_seconds() * arc.speed;

            if arc.angle >= arc.end_angle {
                //TODO: Choose between removing the entity or the component
                // commands.entity(entity).despawn();
                commands.entity(entity).insert(DespawnMarker);
                // commands.entity(entity).remove::<MoveInArc>();
            }
        } else {
            arc.angle -= time.delta_seconds() * arc.speed;
            if arc.angle <= arc.end_angle {
                // commands.entity(entity).despawn();
                commands.entity(entity).insert(DespawnMarker);
                // commands.entity(entity).remove::<MoveInArc>();
            }
        }

        let dir = Vec2::new(
            arc.angle.to_radians().cos(),
            arc.angle.to_radians().sin(),
        )
        // .normalize()
            * arc.radius;

        transform.translation.x = arc.origin.x + dir.x;
        transform.translation.y = arc.origin.y + dir.y;
    }
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component)]
pub struct Rotate {
    pub speed: f32,
    pub to_right: bool,
}

pub fn rotate_system(mut query: Query<(&mut Transform, &Rotate)>, time: Res<Time>) {
    for (mut transform, rotate) in &mut query.iter_mut() {
        let rotation_factor = match rotate.to_right {
            true => -1.,
            false => 1.,
        };

        transform.rotation *=
            Quat::from_rotation_z(rotation_factor * rotate.speed * time.delta_seconds());
    }
}

#[derive(Component)]
pub struct Target {
    pub position: Vec2,
}
pub fn move_to_target(
    mut query: Query<(
        Entity,
        &mut Transform,
        &Stats,
        &Target,
        &mut State,
        &mut Facing,
    )>,
    mut commands: Commands,
    time: Res<Time>,
    mut event_writer: EventWriter<ArrivedEvent>,
) {
    for (entity, mut transform, stats, target, mut state, mut facing) in query.iter_mut() {
        if *state == State::Idle || *state == State::Running {
            let translation_old = transform.translation;
            transform.translation += (target.position.extend(0.) - translation_old).normalize()
                * stats.movement_speed
                * time.delta_seconds();
            if transform.translation.x > translation_old.x {
                *facing = Facing::Right;
            } else {
                *facing = Facing::Left;
            }
            if transform.translation.truncate().distance(target.position) <= 100. {
                commands.entity(entity).remove::<Target>();
                *state = State::Idle;
                event_writer.send(ArrivedEvent(entity))
            } else {
                *state = State::Running;
            }
        }
    }
}

pub fn update_left_movement_boundary(
    query: Query<&Transform, With<Player>>,
    mut boundary: ResMut<LeftMovementBoundary>,
    game_meta: Res<GameMeta>,
) {
    let max_player_x = query
        .iter()
        .map(|transform| transform.translation.x)
        .max_by(|ax, bx| ax.total_cmp(bx));

    if let Some(max_player_x) = max_player_x {
        boundary.0 = boundary
            .0
            .max(max_player_x - game_meta.camera_move_right_boundary - LEFT_BOUNDARY_MAX_DISTANCE);
    }
}

/// Processes the incoming movement commands (e.g. clamps them), and apply them.
///
pub fn process_and_apply_player_movements(
    mut move_commands: EventReader<PlayerMovement>,
    mut players_query: Query<(Entity, &mut Transform, &mut Facing, &mut State), With<Player>>,
    left_movement_boundary: Res<LeftMovementBoundary>,
    game_meta: Res<GameMeta>,
) {
    // First, we collection the locations

    // Hash: {player_id => (Option<movement>, set_facing_state)}
    //
    let mut player_movements = players_query
        .iter()
        .map(|(player_id, _, _, _)| (player_id, (None, false)))
        .collect::<HashMap<_, _>>();

    // Then, we perform the absolute clamping (screen limits), and we collect the data required for
    // the relative clamping.

    let mut min_new_player_x = f32::MAX;

    for PlayerMovement {
        player_id,
        movement,
        set_facing_state,
    } in move_commands.iter()
    {
        let mut movement = *movement;
        let player_movement = player_movements.get_mut(player_id).unwrap();

        let location = players_query.get(*player_id).unwrap().1.translation;

        let new_x = location.x + movement.x;

        if new_x < left_movement_boundary.0 {
            movement.x = 0.;
        }

        //Restrict player to the ground
        let new_y = location.y + movement.y + consts::GROUND_OFFSET;

        if new_y >= consts::MAX_Y || new_y <= consts::MIN_Y {
            movement.y = 0.;
        }

        min_new_player_x = min_new_player_x.min(new_x);

        *player_movement = (Some(movement), *set_facing_state);
    }

    // Then, we perform the clamping of the players relative to each other.

    let max_players_x_distance = LEFT_BOUNDARY_MAX_DISTANCE + game_meta.camera_move_right_boundary;

    for (player_id, location, _, _) in players_query.iter_mut() {
        let movement = player_movements.get_mut(&player_id).unwrap().0.as_mut();

        if let Some(movement) = movement {
            if location.translation.x + movement.x > min_new_player_x + max_players_x_distance {
                *movement = Vec2::ZERO;
            }
        }
    }

    // Now can apply moves, and optionally set the facing direction.
    //
    // This could be merged into the previous step, but it's cleaner this way.

    for (player_id, mut transform, mut facing, mut state) in players_query.iter_mut() {
        let (movement, set_facing_state) = player_movements[&player_id];

        if let Some(movement) = movement {
            transform.translation += movement.extend(0.);

            if set_facing_state {
                if movement.x < 0. {
                    facing.set(Facing::Left);
                } else if movement.x > 0. {
                    facing.set(Facing::Right);
                }

                if movement == Vec2::ZERO {
                    state.set(State::Idle);
                } else {
                    state.set(State::Running);
                }
            }
        }
    }
}
