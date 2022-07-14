use std::path::{Path, PathBuf};

use bevy::{
    asset::{Asset, AssetLoader, AssetPath, LoadedAsset},
    prelude::AddAsset,
    prelude::*,
    reflect::TypeUuid,
};
use bevy_egui::egui;

use crate::metadata::*;

/// Register game asset and loaders
pub fn register(app: &mut bevy::prelude::App) {
    app.register_type::<TextureAtlasSprite>()
        .add_asset::<GameMeta>()
        .add_asset_loader(GameMetaLoader)
        .add_asset::<LevelMeta>()
        .add_asset_loader(LevelMetaLoader)
        .add_asset::<FighterMeta>()
        .add_asset_loader(FighterLoader)
        .add_asset::<EguiFont>()
        .add_asset_loader(EguiFontLoader);
}

/// Calculate an asset's full path relative to another asset
fn relative_asset_path(asset_path: &Path, relative_path: &str) -> PathBuf {
    let is_relative = !relative_path.starts_with('/');

    if is_relative {
        let base = asset_path.parent().unwrap_or_else(|| Path::new(""));
        base.join(relative_path)
    } else {
        Path::new(relative_path)
            .strip_prefix("/")
            .unwrap()
            .to_owned()
    }
}

/// Helper to get relative asset paths and handles
fn get_relative_asset<T: Asset>(
    load_context: &mut bevy::asset::LoadContext,
    self_path: &Path,
    relative_path: &str,
) -> (AssetPath<'static>, Handle<T>) {
    let asset_path = relative_asset_path(self_path, relative_path);
    let asset_path = AssetPath::new(asset_path, None);
    let handle = load_context.get_handle(asset_path.clone());

    (asset_path, handle)
}

#[derive(Default)]
pub struct GameMetaLoader;

impl AssetLoader for GameMetaLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let mut meta: GameMeta = serde_yaml::from_slice(bytes)?;
            trace!(?meta, "Loaded game asset");

            let self_path = load_context.path().to_owned();

            // Detect the system locale
            let locale = sys_locale::get_locale()
                .unwrap_or_else(|| "en-US".to_string())
                .parse()?;
            debug!("Detected system locale: {}", locale);
            meta.translations.detected_locale = locale;

            // Get locale handles
            let mut locale_paths = Vec::new();
            for locale in &meta.translations.locales {
                let (path, handle) = get_relative_asset(load_context, &self_path, locale);
                locale_paths.push(path);
                meta.translations.locale_handles.push(handle);
            }

            // Load the start level asset
            let (start_level_path, start_level_handle) =
                get_relative_asset(load_context, &self_path, &meta.start_level);
            meta.start_level_handle = start_level_handle;

            // Load the main menu background
            let (main_menu_background_path, main_menu_background) = get_relative_asset(
                load_context,
                &self_path,
                &meta.main_menu.background_image.image,
            );
            meta.main_menu.background_image.image_handle = main_menu_background;

            // Load UI fonts
            let mut font_paths = Vec::new();
            for (font_name, font_relative_path) in &meta.ui_theme.font_families {
                let (font_path, font_handle) =
                    get_relative_asset(load_context, &self_path, font_relative_path);

                font_paths.push(font_path);

                meta.ui_theme
                    .font_handles
                    .insert(font_name.clone(), font_handle);
            }

            load_context.set_default_asset(
                LoadedAsset::new(meta)
                    .with_dependencies(vec![start_level_path, main_menu_background_path])
                    .with_dependencies(locale_paths)
                    .with_dependencies(font_paths),
            );

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["game.yml", "game.yaml"]
    }
}

pub struct LevelMetaLoader;

impl AssetLoader for LevelMetaLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let mut meta: LevelMeta = serde_yaml::from_slice(bytes)?;
            trace!(?meta, "Loaded level asset");

            let self_path = load_context.path();

            // Convert all parallax paths to relative asset paths so that the convention matches the
            // rest of the paths used by the asset loaders.
            for layer in &mut meta.parallax_background.layers {
                layer.path = relative_asset_path(self_path, &layer.path)
                    .to_str()
                    .unwrap()
                    .to_owned();
            }

            // Load the players
            let mut dependencies = Vec::new();
            for player in &mut meta.players {
                let player_fighter_file_path = relative_asset_path(self_path, &player.fighter);
                let player_fighter_path = AssetPath::new(player_fighter_file_path.clone(), None);
                let player_fighter_handle = load_context.get_handle(player_fighter_path.clone());
                dependencies.push(player_fighter_path);

                player.fighter_handle = player_fighter_handle;
            }

            // Load the enemies
            for enemy in &mut meta.enemies {
                let enemy_fighter_file_path = relative_asset_path(self_path, &enemy.fighter);
                let enemy_fighter_path = AssetPath::new(enemy_fighter_file_path.clone(), None);
                let enemy_fighter_handle = load_context.get_handle(enemy_fighter_path.clone());
                dependencies.push(enemy_fighter_path);

                enemy.fighter_handle = enemy_fighter_handle;
            }

            load_context.set_default_asset(LoadedAsset::new(meta).with_dependencies(dependencies));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["level.yml", "level.yaml"]
    }
}

pub struct FighterLoader;

impl AssetLoader for FighterLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let mut meta: FighterMeta = serde_yaml::from_slice(bytes)?;
            trace!(?meta, "Loaded fighter asset");

            let self_path = load_context.path();

            let portrait_path = relative_asset_path(self_path, &meta.hud.portrait.image);
            let portrait_path = AssetPath::new(portrait_path, None);
            let portrait_handle = load_context.get_handle(portrait_path.clone());
            meta.hud.portrait.image_handle = portrait_handle;

            let texture_path = relative_asset_path(self_path, &meta.spritesheet.image);
            let texture_path = AssetPath::new(texture_path, None);
            let texture_handle = load_context.get_handle(texture_path.clone());
            let atlas_handle = load_context.set_labeled_asset(
                "atlas",
                LoadedAsset::new(TextureAtlas::from_grid(
                    texture_handle,
                    meta.spritesheet.tile_size.as_vec2(),
                    meta.spritesheet.columns,
                    meta.spritesheet.rows,
                ))
                .with_dependency(texture_path),
            );
            meta.spritesheet.atlas_handle = atlas_handle;

            load_context.set_default_asset(LoadedAsset::new(meta).with_dependency(portrait_path));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["fighter.yml", "fighter.yaml"]
    }
}

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "da277340-574f-4069-907c-7571b8756200"]
pub struct EguiFont(pub egui::FontData);

pub struct EguiFontLoader;

impl AssetLoader for EguiFontLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let path = load_context.path();
            let data = egui::FontData::from_owned(bytes.to_vec());
            trace!(?path, "Loaded font asset");

            load_context.set_default_asset(LoadedAsset::new(EguiFont(data)));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ttf"]
    }
}
