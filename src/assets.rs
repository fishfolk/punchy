use std::path::{Path, PathBuf};

use bevy::{
    asset::{AssetLoader, AssetPath, LoadedAsset},
    prelude::AddAsset,
    prelude::*,
};

use crate::metadata::*;

pub fn register(app: &mut bevy::prelude::App) {
    app.register_type::<TextureAtlasSprite>()
        .add_asset::<Game>()
        .add_asset_loader(GameLoader)
        .add_asset::<Level>()
        .add_asset_loader(LevelLoader)
        .add_asset::<Fighter>()
        .add_asset_loader(FighterLoader);
}

#[derive(thiserror::Error, Debug)]
pub enum AssetLoaderError {
    #[error("Could not parse YAML asset: {0}")]
    DeserializationError(#[from] serde_yaml::Error),
}

fn relative_asset_path(asset_path: &Path, relative: &str) -> PathBuf {
    let is_relative = !relative.starts_with('/');

    if is_relative {
        let base = asset_path.parent().unwrap_or_else(|| Path::new(""));
        base.join(relative)
    } else {
        Path::new(relative).strip_prefix("/").unwrap().to_owned()
    }
}

#[derive(Default)]
pub struct GameLoader;

impl AssetLoader for GameLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let meta: GameMeta = serde_yaml::from_slice(bytes)?;
            trace!(?meta, "Loaded game asset");

            let self_path = load_context.path();

            let start_level_path = relative_asset_path(self_path, &meta.start_level);
            let start_level_path = AssetPath::new(start_level_path, None);
            let start_level = load_context.get_handle(start_level_path.clone());

            load_context.set_default_asset(
                LoadedAsset::new(Game { meta, start_level }).with_dependency(start_level_path),
            );

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["game.yml", "game.yaml"]
    }
}

pub struct LevelLoader;

impl AssetLoader for LevelLoader {
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

            let player_fighter_file_path =
                relative_asset_path(self_path, &meta.player_spawn.fighter);
            let player_fighter_path = AssetPath::new(player_fighter_file_path, None);
            let player_fighter_handle = load_context.get_handle(player_fighter_path.clone());

            let mut enemy_fighter_handles = Vec::new();
            let mut enemy_asset_paths = Vec::new();

            for enemy in &meta.enemies {
                let enemy_fighter_file_path = relative_asset_path(self_path, &enemy.fighter);
                let enemy_fighter_path = AssetPath::new(enemy_fighter_file_path.clone(), None);
                let enemy_fighter_handle = load_context.get_handle(enemy_fighter_path.clone());
                enemy_asset_paths.push(enemy_fighter_path);

                enemy_fighter_handles.push(enemy_fighter_handle);
            }

            load_context.set_default_asset(
                LoadedAsset::new(Level {
                    meta,
                    player_fighter_handle,
                    enemy_fighter_handles,
                })
                .with_dependency(player_fighter_path)
                .with_dependencies(enemy_asset_paths),
            );

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
            let meta: FighterMeta = serde_yaml::from_slice(bytes)?;
            trace!(?meta, "Loaded fighter asset");

            let self_path = load_context.path();
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

            load_context.set_default_asset(LoadedAsset::new(Fighter { meta, atlas_handle }));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["fighter.yml", "fighter.yaml"]
    }
}
