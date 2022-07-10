use bevy::{
    core::{Time, Timer},
    math::{Quat, Vec2, Vec3Swizzles},
    prelude::{
        Commands, Component, Deref, DerefMut, Entity, EventWriter, Query, Res, ResMut, Transform,
        With,
    },
};
use leafwing_input_manager::prelude::ActionState;

use crate::{
    animation::Facing,
    consts::{self, LEFT_BOUNDARY_MAX_DISTANCE},
    input::PlayerAction,
    item::ThrowItemEvent,
    metadata::GameMeta,
    state::State,
    ArrivedEvent, DespawnMarker, Player, Stats,
};

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Deref, DerefMut)]
pub struct MoveInDirection(pub Vec2);

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
    mut query: Query<
        (
            &mut State,
            &Stats,
            &mut Transform,
            Option<&mut Facing>,
            &ActionState<PlayerAction>,
        ),
        With<Player>,
    >,
    time: Res<Time>,
    game_meta: Res<GameMeta>,
    left_movement_boundary: Res<LeftMovementBoundary>,
) {
    let players_x = query
        .iter()
        .map(|(_, _, transform, _, _)| transform.translation.x)
        .collect::<Vec<_>>();

    // Compute the new direction vectors; can be None if the state is not (idle or running).
    //
    let mut player_dirs = query
        .iter()
        .map(|(state, stats, transform, _, input)| {
            if *state != State::Idle && *state != State::Running {
                None
            } else {
                let mut dir = Vec2::ZERO;

                if input.pressed(PlayerAction::Move) {
                    dir = input.action_axis_pair(PlayerAction::Move).unwrap().xy();
                }

                // Apply speed
                dir = dir * stats.movement_speed * time.delta_seconds();

                let new_x = transform.translation.x + dir.x;

                // The dir.x condition allows some flexibility (e.g. in case of knockback), given
                // the current state of development. To be removed once the movement logic is
                // stabilized.
                //
                if dir.x < 0. && new_x < left_movement_boundary.0 {
                    dir.x = 0.;
                }

                //Restrict player to the ground
                let new_y = transform.translation.y + dir.y + consts::GROUND_OFFSET;

                if new_y >= consts::MAX_Y || new_y <= consts::MIN_Y {
                    dir.y = 0.;
                }

                //Move the player
                Some(dir)
            }
        })
        .collect::<Vec<_>>();

    if player_dirs.len() > 1 {
        let max_players_x_distance =
            LEFT_BOUNDARY_MAX_DISTANCE + game_meta.camera_move_right_boundary;

        let new_players_x = players_x
            .iter()
            .zip(player_dirs.iter())
            .map(|(x, dir)| x + dir.unwrap_or(Vec2::ZERO).x)
            .collect::<Vec<_>>();

        let min_player_x = new_players_x
            .iter()
            .min_by(|ax, bx| ax.total_cmp(bx))
            .unwrap();

        for (player_dir, player_x) in player_dirs.iter_mut().zip(new_players_x.iter()) {
            if let Some(player_dir) = player_dir.as_mut() {
                if *player_x > min_player_x + max_players_x_distance {
                    *player_dir = Vec2::ZERO;
                }
            }
        }
    }

    for ((mut state, _, mut transform, facing_option, _), dir) in
        query.iter_mut().zip(player_dirs.iter())
    {
        if let Some(dir) = dir {
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

            if dir == &Vec2::ZERO {
                state.set(State::Idle);
            } else {
                state.set(State::Running);
            }
        }
    }
}

pub fn throw_item_system(
    query: Query<(&Transform, Option<&Facing>, &ActionState<PlayerAction>), With<Player>>,
    mut ev_throw_item: EventWriter<ThrowItemEvent>,
) {
    for (transform, facing_option, input) in query.iter() {
        if input.just_pressed(PlayerAction::Throw) {
            let facing = match facing_option {
                Some(f) => f.clone(),
                None => Facing::Right,
            };

            let mut position = transform.translation.xy();

            //Offset the position depending on the facing
            if facing.is_left() {
                position.x -= consts::THROW_ITEM_X_OFFSET;
            } else {
                position.x += consts::THROW_ITEM_X_OFFSET;
            }

            position.y -= consts::PLAYER_HEIGHT / 2.; //Set to the player feet

            ev_throw_item.send(ThrowItemEvent {
                position,
                facing: facing.clone(),
            })
        }
    }
}

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
) {
    for transform in query.iter() {
        boundary.0 = boundary
            .0
            .max(transform.translation.x - LEFT_BOUNDARY_MAX_DISTANCE)
    }
}
