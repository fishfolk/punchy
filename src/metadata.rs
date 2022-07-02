use bevy::{
    math::{UVec2, Vec2, Vec3},
    prelude::{Color, Component, Handle, Image},
    reflect::TypeUuid,
    sprite::TextureAtlas,
    utils::HashMap,
};
use bevy_egui::egui;
use bevy_parallax::{LayerData, ParallaxResource};
use serde::Deserialize;

use crate::{animation::Clip, assets::EguiFont, state::State, Stats};

#[derive(TypeUuid, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
#[uuid = "eb28180f-ef68-44a0-8479-a299a3cef66e"]
pub struct GameMeta {
    pub start_level: String,
    #[serde(skip)]
    pub start_level_handle: Handle<LevelMeta>,
    pub main_menu: MainMenuMeta,
    pub ui_theme: UIThemeMeta,
    pub camera_height: u32,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MainMenuMeta {
    pub title: String,
    pub title_size: f32,
    pub title_font: String,
    pub background_image: ImageMeta,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ImageMeta {
    pub image: String,
    pub size: Vec2,
    #[serde(skip)]
    pub handle: Handle<Image>,
}

#[derive(TypeUuid, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
#[uuid = "32111f6e-bb9a-4ea7-8988-1220b923a059"]
pub struct LevelMeta {
    pub background_color: [u8; 3],
    pub parallax_background: ParallaxMeta,
    pub player: FighterSpawnMeta,
    #[serde(default)]
    pub enemies: Vec<FighterSpawnMeta>,
}

impl LevelMeta {
    pub fn background_color(&self) -> Color {
        let [r, g, b] = self.background_color;
        Color::rgb_u8(r, g, b)
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
    pub animations: HashMap<State, Clip>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct FighterSpawnMeta {
    pub fighter: String,
    #[serde(skip)]
    pub fighter_handle: Handle<Fighter>,
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

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct UIThemeMeta {
    pub fonts: HashMap<String, String>,
    #[serde(skip)]
    pub font_handles: HashMap<String, Handle<EguiFont>>,
    pub panel: UIPanelThemeMeta,
    pub button: UIButtonThemeMeta,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct UIPanelThemeMeta {
    #[serde(default)]
    pub text_color: ColorMeta,
    #[serde(default)]
    pub padding: MarginMeta,
    pub border: UIBorderImageMeta,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct UIButtonThemeMeta {
    #[serde(default)]
    pub text_color: ColorMeta,
    pub font_size: f32,
    pub font: String,
    #[serde(default)]
    pub padding: MarginMeta,
    pub borders: UIButtonBordersMeta,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct UIBorderImageMeta {
    pub image: String,
    pub image_size: UVec2,
    pub border_size: MarginMeta,
    #[serde(default = "f32_one")]
    pub scale: f32,
    #[serde(default)]
    pub only_frame: bool,

    #[serde(skip)]
    pub handle: Handle<Image>,
    #[serde(skip)]
    pub egui_texture: egui::TextureId,
}

fn f32_one() -> f32 {
    1.0
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct UIButtonBordersMeta {
    pub default: UIBorderImageMeta,
    #[serde(default)]
    pub hovered: Option<UIBorderImageMeta>,
    #[serde(default)]
    pub clicked: Option<UIBorderImageMeta>,
}

#[derive(Default, Deserialize, Clone, Copy, Debug)]
#[serde(deny_unknown_fields)]
pub struct ColorMeta([u8; 3]);

impl From<ColorMeta> for egui::Color32 {
    fn from(c: ColorMeta) -> Self {
        let [r, g, b] = c.0;
        egui::Color32::from_rgb(r, g, b)
    }
}

#[derive(Default, Deserialize, Clone, Copy, Debug)]
#[serde(deny_unknown_fields)]
pub struct MarginMeta {
    #[serde(default)]
    pub top: f32,
    #[serde(default)]
    pub bottom: f32,
    #[serde(default)]
    pub left: f32,
    #[serde(default)]
    pub right: f32,
}

impl From<MarginMeta> for bevy_egui::egui::style::Margin {
    fn from(m: MarginMeta) -> Self {
        Self {
            left: m.left,
            right: m.right,
            top: m.top,
            bottom: m.bottom,
        }
    }
}
