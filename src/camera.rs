use bevy::{
    math::Vec2,
    prelude::{Camera, Component, EventWriter, Query, Res, Transform, With, Without},
};
use bevy_parallax::ParallaxMoveEvent;

use crate::{consts, metadata::GameMeta, Player};

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component)]
pub struct Panning {
    pub offset: Vec2,
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
