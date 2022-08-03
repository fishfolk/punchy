use bevy::{
    math::{Quat, Vec2},
    prelude::*,
    time::{Time, Timer},
};
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    animation::Facing,
    consts::{self, LEFT_BOUNDARY_MAX_DISTANCE},
    enemy::SpawnLocationX,
    input::PlayerAction,
    metadata::{GameMeta, LevelMeta},
    ArrivedEvent, DespawnMarker, GameState, Player, Stats,
};

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            // Here we add a chain of systems that act as constraints on movements, ending the chain
            // with the velocity system itself which applies the velocities to the entities.
            update_left_movement_boundary
                .chain(constrain_player_movement)
                .chain(velocity_system)
                .run_in_state(GameState::InGame),
        );
    }
}

/// An entity's velocity.
///
/// This is similar to the velocity you would set in a physics simulation, but in our case we use a
/// simple constraints system instead of actual physics simulation.
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Deref, DerefMut, Default)]
pub struct Velocity(pub Vec2);

/// System that updates translations based on entity velocities.
pub fn velocity_system(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, dir) in &mut query.iter_mut() {
        transform.translation += dir.0.extend(0.) * time.delta_seconds();
    }
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
    // player_movement_clamper: PlayerMovementClamper,
    time: Res<Time>,
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

    let (mut enemy_knockbacks, mut player_knockbacks): (Vec<_>, Vec<_>) = current_knockbacks
        .iter_mut()
        .partition(|(_, _, _, player)| player.is_none());

    for (_, transform, knockback, _) in enemy_knockbacks.iter_mut() {
        transform.translation.x += knockback.direction.x * time.delta_seconds();
        transform.translation.y += knockback.direction.y * time.delta_seconds();
    }

    // Extract the players movement data, and apply the knockbacks, clamped.

    let player_movements = player_knockbacks
        .iter()
        .map(|(_, transform, knockback, _)| {
            (
                transform.translation,
                Some(knockback.direction * time.delta_seconds()),
            )
        })
        .collect::<Vec<_>>();

    // let player_dirs = player_movement_clamper.clamp(player_movements);

    // for ((_, transform, _, _), player_dir) in player_knockbacks.iter_mut().zip(player_dirs) {
    //     transform.translation += player_dir.unwrap().extend(0.);
    // }
}

pub fn player_controller(
    mut query: Query<
        (
            &Stats,
            &mut Transform,
            &mut Facing,
            &ActionState<PlayerAction>,
        ),
        With<Player>,
    >,
    // player_movement_clamper: PlayerMovementClamper,
    time: Res<Time>,
) {
    // // Compute the new direction vectors; can be None if the state is not (idle or running).
    // //
    // let player_movements = query
    //     .iter()
    //     .map(|(state, stats, transform, _, input)| {
    //         if *state != State::Idle && *state != State::Running {
    //             (transform.translation, None)
    //         } else {
    //             let mut dir = if input.pressed(PlayerAction::Move) {
    //                 input.axis_pair(PlayerAction::Move).unwrap().xy()
    //             } else {
    //                 Vec2::ZERO
    //             };

    //             // Apply speed
    //             dir = dir * stats.movement_speed * time.delta_seconds();

    //             //Move the player
    //             (transform.translation, Some(dir))
    //         }
    //     })
    //     .collect::<Vec<_>>();

    // let player_dirs = player_movement_clamper.clamp(player_movements);

    // for ((mut state, _, mut transform, mut facing, _), dir) in
    //     query.iter_mut().zip(player_dirs.iter())
    // {
    //     if let Some(dir) = dir {
    //         transform.translation.x += dir.x;
    //         transform.translation.y += dir.y;

    //         //Set the player state and direction
    //         if dir.x < 0. {
    //             facing.set(Facing::Left);
    //         } else if dir.x > 0. {
    //             facing.set(Facing::Right);
    //         }

    //         if dir == &Vec2::ZERO {
    //             state.set(State::Idle);
    //         } else {
    //             state.set(State::Running);
    //         }
    //     }
    // }
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

/// Target belonging to an enemy (therefore, the player).
/// The attack distance is for randomization purposes, and it's the distance that triggers the
/// attack. More precisely, it's the max distance - if the enemy finds itself at a smaller
/// distance, it will attack.
#[derive(Component)]
pub struct Target {
    pub position: Vec2,
    pub attack_distance: f32,
}

pub fn move_to_target(
    mut query: Query<(Entity, &mut Transform, &Stats, &Target, &mut Facing)>,
    mut commands: Commands,
    time: Res<Time>,
    mut event_writer: EventWriter<ArrivedEvent>,
) {
    // for (entity, mut transform, stats, target, mut state, mut facing) in query.iter_mut() {
    //     if *state == State::Idle || *state == State::Running {
    //         let translation_old = transform.translation;
    //         transform.translation += (target.position.extend(0.) - translation_old).normalize()
    //             * stats.movement_speed
    //             * time.delta_seconds();
    //         if transform.translation.x > translation_old.x {
    //             *facing = Facing::Right;
    //         } else {
    //             *facing = Facing::Left;
    //         }
    //         if transform.translation.truncate().distance(target.position) <= 100. {
    //             commands.entity(entity).remove::<Target>();
    //             *state = State::Idle;
    //             event_writer.send(ArrivedEvent(entity))
    //         } else {
    //             *state = State::Running;
    //         }
    //     }
    // }
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

fn constrain_player_movement(
    enemy_spawn_locations_query: Query<&'static SpawnLocationX>,
    level_meta: Res<LevelMeta>,
    game_meta: Res<GameMeta>,
    left_movement_boundary: Res<LeftMovementBoundary>,
    mut players: Query<(&Transform, &mut Velocity), With<Player>>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();

    // Collect player positions and velocities
    let mut player_velocities = players
        .iter_mut()
        .map(|(transform, vel)| (transform.translation, vel))
        .collect::<Vec<_>>();

    // Identify the current stop poing
    let current_stop_point = level_meta.stop_points.iter().find(|point_x| {
        player_velocities
            .iter()
            .any(|(location, dir)| location.x < **point_x && **point_x <= location.x + dir.x)
    });

    // If there is a current stop point
    if let Some(current_stop_point) = current_stop_point {
        let any_enemy_behind_stop_point = enemy_spawn_locations_query
            .iter()
            .any(|SpawnLocationX(spawn_x)| spawn_x <= current_stop_point);

        // Prevent movement beyond the stop point if there are enemies not yet defeated behind the
        // stop point.
        if any_enemy_behind_stop_point {
            for (location, velocity) in player_velocities.iter_mut() {
                // Can be simplified, but it's harder to understand.
                if location.x + velocity.x * dt > *current_stop_point {
                    velocity.x = 0.;
                }
            }
        }
    }

    // Then, we perform the absolute clamping (screen top/left/bottom), and we collect the data
    // required for the relative clamping.

    let mut min_new_player_x = f32::MAX;

    #[allow(clippy::needless_collect)] // False alarm
    let velocities = player_velocities
        .into_iter()
        .map(|(location, mut velocity)| {
            let new_x = location.x + velocity.x * dt;

            if new_x < left_movement_boundary.0 {
                velocity.x = 0.;
            }

            //Restrict player to the ground
            let new_y = location.y + velocity.y * dt + consts::GROUND_OFFSET;

            if new_y >= consts::MAX_Y || new_y <= consts::MIN_Y {
                velocity.y = 0.;
            }

            let new_velocity = (velocity, new_x);

            min_new_player_x = min_new_player_x.min(new_x);

            (location, new_velocity)
        })
        .collect::<Vec<_>>();

    // Then, we perform the clamping of the players relative to each other.
    let max_players_x_distance = LEFT_BOUNDARY_MAX_DISTANCE + game_meta.camera_move_right_boundary;

    velocities
        .into_iter()
        .for_each(|(_, (mut velocity, new_player_x))| {
            if new_player_x > min_new_player_x + max_players_x_distance {
                **velocity = Vec2::ZERO
            }
        });
}
