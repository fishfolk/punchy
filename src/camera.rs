use bevy::{
    core::Time,
    input::Input,
    math::Vec2,
    prelude::{
        Camera, Component, EventWriter, KeyCode, OrthographicProjection, Query, Res, ResMut,
        Transform, With, Without,
    },
    render::camera::CameraProjection,
    window::Windows,
};
use bevy_parallax::ParallaxMoveEvent;

use crate::{consts, Player};

#[derive(Component)]
pub struct Panning {
    pub offset: Vec2,
}

pub fn helper_camera_controller(
    mut query: Query<(&mut Camera, &mut OrthographicProjection, &mut Panning)>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut windows: ResMut<Windows>,
) {
    let (mut camera, mut projection, mut panning) = query.single_mut();

    if keys.pressed(KeyCode::Up) {
        panning.offset.y += 150.0 * time.delta_seconds();
    }
    if keys.pressed(KeyCode::Left) {
        panning.offset.x -= 150.0 * time.delta_seconds();
    }
    if keys.pressed(KeyCode::Down) {
        panning.offset.y -= 150.0 * time.delta_seconds();
    }
    if keys.pressed(KeyCode::Right) {
        panning.offset.x += 150.0 * time.delta_seconds();
    }

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
    if let Ok(player) = player_query.get_single() {
        let (mut camera, panning) = camera_query.single_mut();

        let diff = player.translation.x - (camera.translation.x - panning.offset.x);

        camera.translation.x = player.translation.x + panning.offset.x;
        camera.translation.y = consts::GROUND_Y + panning.offset.y;

        move_event_writer.send(ParallaxMoveEvent {
            camera_move_speed: diff * consts::CAMERA_SPEED,
        });
    }
}
