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

use crate::{consts, input::CameraAction, Player};

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

pub fn camera_follow_player(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<(&mut Transform, &Panning), (With<Camera>, Without<Player>)>,
    mut move_event_writer: EventWriter<ParallaxMoveEvent>,
) {
    // TODO: Follow both players, not just the first one
    if let Some(player) = player_query.iter().next() {
        let (mut camera, panning) = camera_query.single_mut();

        let diff = player.translation.x - (camera.translation.x - panning.offset.x);

        camera.translation.x = player.translation.x + panning.offset.x;
        camera.translation.y = consts::GROUND_Y + panning.offset.y;

        move_event_writer.send(ParallaxMoveEvent {
            camera_move_speed: diff * consts::CAMERA_SPEED,
        });
    }
}
