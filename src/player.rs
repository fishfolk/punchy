use bevy::prelude::*;
use leafwing_input_manager::InputManagerBundle;

use crate::{
    animation::Facing,
    consts,
    input::PlayerAction,
    metadata::{FighterMeta, FighterSpawnMeta, GameMeta},
};

#[derive(Component)]
pub struct Player;

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    facing: Facing,
    #[bundle]
    transform_bundle: TransformBundle,
    fighter_handle: Handle<FighterMeta>,
    #[bundle]
    input_manager_bundle: InputManagerBundle<PlayerAction>,
}

impl PlayerBundle {
    pub fn new(player_meta: &FighterSpawnMeta, player_i: usize, game_meta: &GameMeta) -> Self {
        let ground_offset = Vec3::new(0.0, consts::GROUND_Y, 0.0);
        let player_pos = player_meta.location + ground_offset;

        let transform_bundle =
            TransformBundle::from_transform(Transform::from_translation(player_pos));

        let fighter_handle = player_meta.fighter_handle.clone();

        let input_manager_bundle = InputManagerBundle {
            input_map: game_meta
                .default_input_maps
                .get_player_map(player_i)
                .map(|mut map| map.set_gamepad(Gamepad(player_i)).build())
                .unwrap_or_default(),
            ..default()
        };

        PlayerBundle {
            player: Player,
            facing: Facing::Right,
            transform_bundle,
            fighter_handle,
            input_manager_bundle,
        }
    }
}
