use bevy::prelude::*;
use bevy_parallax::ParallaxMoveEvent;
use iyes_loopless::prelude::*;

use crate::{consts, metadata::GameMeta, movement::VelocitySystems, GameState, Player};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register reflect types
            .register_type::<YSort>()
            // Add systems
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .after(VelocitySystems)
                    .with_system(camera_follow_player)
                    .with_system(y_sort)
                    .into(),
            );
    }
}

/// Component to sort entities by their y position.
/// Takes in a offset usually the sprite's height.
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct YSort(pub f32);

/// Applies the y-sorting to the entities Z position.
pub fn y_sort(mut query: Query<(&mut Transform, &YSort)>) {
    for (mut transform, ysort) in query.iter_mut() {
        transform.translation.z = ysort.0 - transform.translation.y;
    }
}

/// Moves the camera according to the RIGHT_BOUNDARY_DISTANCE. Note that this does not enforce
/// limitations of any kind - that's up to the players movement logic (e.g. max distance).
pub fn camera_follow_player(
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<&Transform, (With<Camera>, Without<Player>)>,
    mut move_event_writer: EventWriter<ParallaxMoveEvent>,
    game_meta: Res<GameMeta>,
) {
    let max_player_x = player_query
        .iter()
        .map(|transform| transform.translation.x)
        .max_by(|ax, bx| ax.total_cmp(bx));

    if let Some(max_player_x) = max_player_x {
        let camera = camera_query.single();

        let max_player_x_diff =
            max_player_x - camera.translation.x - game_meta.camera_move_right_boundary;

        if max_player_x_diff > 0. {
            // The x axis is handled by the parallax plugin.
            // The y axis value doesn't change.

            move_event_writer.send(ParallaxMoveEvent {
                camera_move_speed: max_player_x_diff * consts::CAMERA_SPEED,
            });
        }
    }
}
