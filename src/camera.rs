use bevy::{
    core::Time,
    math::Vec2,
    prelude::{
        Camera, Component, EventWriter, OrthographicProjection, Query, Res, ResMut, Transform,
        With, Without,
    },
    render::camera::CameraProjection,
    window::Windows,
};
use bevy_parallax::ParallaxMoveEvent;
use leafwing_input_manager::prelude::ActionState;

use crate::{consts, input::CameraAction, metadata::GameMeta, Player};

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component)]
pub struct Panning {
    pub offset: Vec2,
}

pub fn helper_camera_controller(
    mut query: Query<(
        &mut Camera,
        &mut OrthographicProjection,
        &mut Panning,
        &ActionState<CameraAction>,
    )>,
    time: Res<Time>,
    mut windows: ResMut<Windows>,
) {
    let (mut camera, mut projection, mut panning, input) = query.single_mut();

    use CameraAction::*;
    if input.pressed(Up) {
        panning.offset.y += 150.0 * time.delta_seconds();
    }
    if input.pressed(Left) {
        panning.offset.x -= 150.0 * time.delta_seconds();
    }
    if input.pressed(Down) {
        panning.offset.y -= 150.0 * time.delta_seconds();
    }
    if input.pressed(Right) {
        panning.offset.x += 150.0 * time.delta_seconds();
    }

    if input.pressed(ZoomIn) {
        projection.scale = f32::clamp(
            projection.scale - 150. * time.delta_seconds(),
            1.,
            projection.scale,
        );
    }
    if input.pressed(ZoomOut) {
        projection.scale += 150. * time.delta_seconds();
    }

    let scale = projection.scale;
    let window = windows.primary_mut();

    if (projection.scale - scale).abs() > f32::EPSILON {
        projection.update(window.width(), window.height());
        camera.projection_matrix = projection.get_projection_matrix();
        camera.depth_calculation = projection.depth_calculation();
    }
}

/// Moves the camera according to the RIGHT_BOUNDARY_DISTANCE. Note that this does not enforce
/// limitations of any kind - that's up to the players movement logic (e.g. max distance).
pub fn camera_follow_player(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<(&mut Transform, &Panning), (With<Camera>, Without<Player>)>,
    mut move_event_writer: EventWriter<ParallaxMoveEvent>,
    game_meta: Res<GameMeta>,
) {
    let max_player_x = player_query
        .iter()
        .map(|transform| transform.translation.x)
        .max_by(|ax, bx| ax.total_cmp(bx));

    if let Some(max_player_x) = max_player_x {
        let (mut camera, panning) = camera_query.single_mut();

        let max_player_x_diff =
            max_player_x - camera.translation.x - game_meta.camera_move_right_boundary;

        if max_player_x_diff > 0. {
            // The x axis is handled by the parallax plugin.
            camera.translation.y = consts::GROUND_Y + panning.offset.y;

            move_event_writer.send(ParallaxMoveEvent {
                camera_move_speed: max_player_x_diff * consts::CAMERA_SPEED,
            });
        }
    }
}
