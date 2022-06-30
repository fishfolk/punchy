use bevy::{
    math::{UVec2, Vec2, Vec3},
    prelude::{Color, Component, Handle},
    reflect::TypeUuid,
    sprite::TextureAtlas,
    utils::HashMap,
};
use bevy_parallax::{LayerData, ParallaxResource};
use serde::Deserialize;

use crate::{state::State, Stats};

#[derive(TypeUuid, Clone, Debug)]
#[uuid = "eb28180f-ef68-44a0-8479-a299a3cef66e"]
pub struct Game {
    pub meta: GameMeta,
    pub start_level: Handle<Level>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct GameMeta {
    pub start_level: String,
}

#[derive(TypeUuid, Clone, Debug)]
#[uuid = "32111f6e-bb9a-4ea7-8988-1220b923a059"]
pub struct Level {
    pub meta: LevelMeta,
    pub player_fighter_handle: Handle<Fighter>,
    pub enemy_fighter_handles: Vec<Handle<Fighter>>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct LevelMeta {
    pub background_color: [u8; 3],
    pub parallax_background: ParallaxMeta,
    pub player_spawn: FighterSpawnMeta,
    #[serde(default)]
    pub enemies: Vec<FighterSpawnMeta>,
}

impl LevelMeta {
    pub fn background_color(&self) -> Color {
        Color::rgb_u8(
            self.background_color[0],
            self.background_color[1],
            self.background_color[2],
        )
    }
}

#[derive(TypeUuid, Clone, Debug)]
#[uuid = "d5e040c4-3de7-4b8a-b6c2-27f82f58d8f0"]
pub struct Fighter {
    pub meta: FighterMeta,
    pub atlas_handle: Handle<TextureAtlas>,
}

#[derive(Deserialize, Clone, Debug, Component)]
#[serde(deny_unknown_fields)]
pub struct FighterMeta {
    pub name: String,
    pub stats: Stats,
    pub spritesheet: FighterSpritesheetMeta,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct FighterSpritesheetMeta {
    pub image: String,
    pub tile_size: UVec2,
    pub columns: usize,
    pub rows: usize,
    pub animation_fps: f32,
    pub animations: HashMap<State, std::ops::Range<usize>>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct FighterSpawnMeta {
    pub fighter: String,
    pub location: Vec3,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ParallaxMeta {
    pub layers: Vec<ParallaxLayerMeta>,
}

impl ParallaxMeta {
    pub fn get_resource(&self) -> ParallaxResource {
        ParallaxResource::new(self.layers.iter().cloned().map(Into::into).collect())
    }
}

// TODO: This struct is a workaround for the fact that `bevy_parallax::LayerData` isn't Clone.
#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ParallaxLayerMeta {
    pub speed: f32,
    pub path: String,
    pub tile_size: Vec2,
    pub cols: usize,
    pub rows: usize,
    pub scale: f32,
    pub z: f32,
    pub transition_factor: f32,
}

impl From<ParallaxLayerMeta> for LayerData {
    fn from(meta: ParallaxLayerMeta) -> Self {
        Self {
            speed: meta.speed,
            path: meta.path,
            tile_size: meta.tile_size,
            cols: meta.cols,
            rows: meta.rows,
            scale: meta.scale,
            z: meta.z,
            transition_factor: meta.transition_factor,
        }
    }
}
