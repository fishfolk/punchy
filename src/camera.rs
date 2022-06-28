use bevy::{
    core::Time,
    input::Input,
    prelude::{
        Camera, EventWriter, KeyCode, OrthographicProjection, Query, Res, ResMut, Transform, With,
        Without,
    },
    render::camera::CameraProjection,
    window::Windows,
};
use bevy_parallax::ParallaxMoveEvent;

use crate::{consts, Player};

pub fn helper_camera_controller(
    mut query: Query<(&mut Camera, &mut OrthographicProjection, &mut Transform)>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut windows: ResMut<Windows>,
) {
    let (mut camera, mut projection, mut transform) = query.single_mut();

    if keys.pressed(KeyCode::Up) {
        transform.translation.y += 150.0 * time.delta_seconds();
    }
    if keys.pressed(KeyCode::Left) {
        transform.translation.x -= 150.0 * time.delta_seconds();
    }
    if keys.pressed(KeyCode::Down) {
        transform.translation.y -= 150.0 * time.delta_seconds();
    }
    if keys.pressed(KeyCode::Right) {
        transform.translation.x += 150.0 * time.delta_seconds();
    }

    let scale = projection.scale;

    let w = windows.primary_mut();

    if keys.pressed(KeyCode::Z) {
        projection.scale = f32::clamp(
            projection.scale - 150. * time.delta_seconds(),
            1.,
            projection.scale,
        );
    }
    if keys.pressed(KeyCode::X) {
        projection.scale += 150. * time.delta_seconds();
    }

    if (projection.scale - scale).abs() > f32::EPSILON {
        projection.update(w.width(), w.height());
        camera.projection_matrix = projection.get_projection_matrix();
        camera.depth_calculation = projection.depth_calculation();
    }
}

pub fn camera_follow_player(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    mut move_event_writer: EventWriter<ParallaxMoveEvent>,
) {
    let player = player_query.single().translation;
    let camera = camera_query.single_mut();

    move_event_writer.send(ParallaxMoveEvent {
        camera_move_speed: (player.x - camera.translation.x) * consts::CAMERA_SPEED,
    });
    //   camera.translation.y += (player.y - camera.translation.y) * time.delta_seconds() * 5.;
}
