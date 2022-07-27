use bevy::{asset::AssetStage, prelude::*};
use bevy_parallax::ParallaxResource;
use bevy_rapier2d::prelude::CollisionGroups;
use iyes_loopless::{
    prelude::{ConditionSet, IntoConditionalSystem},
    state::NextState,
};

use rand::seq::SliceRandom;

use crate::{
    animation::Animation,
    collisions::BodyLayers,
    config::EngineConfig,
    enemy::{Enemy, EnemyBundle},
    input::MenuAction,
    item::ItemBundle,
    metadata::{BorderImageMeta, FighterMeta, GameMeta, ItemMeta, LevelMeta, Settings},
    platform::Storage,
    player::{Player, PlayerBundle},
    AnimatedSpriteSheetBundle, CharacterBundle, GameStage, GameState, PhysicsBundle, Stats,
};

use bevy::{ecs::system::SystemParam, render::camera::ScalingMode};
use bevy_egui::{egui, EguiContext};
use bevy_fluent::Locale;
use bevy_parallax::ParallaxCameraComponent;
use leafwing_input_manager::{
    axislike::{AxisType, SingleAxis},
    prelude::InputMap,
    InputManagerBundle,
};

use progress::{HasLoadProgress, LoadingResources};
pub mod progress;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_arch = "wasm32"))]
        let engine_config = {
            use structopt::StructOpt;
            EngineConfig::from_args()
        };

        #[cfg(target_arch = "wasm32")]
        let engine_config = EngineConfig::from_web_params();

        app.add_system(load_level.run_in_state(GameState::LoadingLevel))
            .add_system(load_game.run_in_state(GameState::LoadingGame))
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(load_fighters)
                    .with_system(load_items)
                    .into(),
            );

        // Configure hot reload
        if engine_config.hot_reload {
            app.add_stage_after(
                AssetStage::LoadAssets,
                GameStage::HotReload,
                SystemStage::parallel(),
            )
            .add_system_to_stage(GameStage::HotReload, hot_reload_game)
            .add_system_set_to_stage(
                GameStage::HotReload,
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(hot_reload_level)
                    .with_system(hot_reload_fighters)
                    .into(),
            );
        }
    }
}

/// System param used to load and hot reload the game
#[derive(SystemParam)]
pub struct GameLoader<'w, 's> {
    skip_next_asset_update_event: Local<'s, bool>,
    camera: Query<'w, 's, Entity, With<Camera>>,
    commands: Commands<'w, 's>,
    game_handle: Res<'w, Handle<GameMeta>>,
    assets: ResMut<'w, Assets<GameMeta>>,
    egui_ctx: ResMut<'w, EguiContext>,
    events: EventReader<'w, 's, AssetEvent<GameMeta>>,
    loading_resources: LoadingResources<'w, 's>,
}

impl<'w, 's> GameLoader<'w, 's> {
    /// This function is called once when the game starts up and, when hot reload is enabled, on
    /// update, to check for asset changed events and to update the [`GameMeta`] resource.
    ///
    /// The `is_hot_reload` argument is used to indicate whether the function should check for asset
    /// updates and reload, or whether it should run the one-time initialization of the game.
    fn load(mut self, is_hot_reload: bool) {
        // Check to make sure we shouldn't skip this execution
        // ( i.e. if this is a hot reload run without any changed assets )
        if self.should_skip_run(is_hot_reload) {
            return;
        }

        let Self {
            mut skip_next_asset_update_event,
            camera,
            mut commands,
            game_handle,
            mut assets,
            mut egui_ctx,
            ..
        } = self;

        if let Some(game) = assets.get_mut(game_handle.clone_weak()) {
            // Track load progress
            let load_progress = game.load_progress(&self.loading_resources);
            debug!(
                %load_progress,
                "Loading game assets: {:.2}% ",
                load_progress.as_percent()
            );

            // Wait until assets are loaded to start game
            if load_progress.as_percent() < 1.0 {
                return;
            }

            // Hot reload preparation
            if is_hot_reload {
                // Despawn previous camera
                if let Ok(camera) = camera.get_single() {
                    commands.entity(camera).despawn();
                }

                // Since we are modifying the game asset, which will trigger another asset changed
                // event, we need to skip the next update event.
                *skip_next_asset_update_event = true;

            // One-time initialization
            } else {
                // Initialize empty fonts for all game fonts.
                //
                // This makes sure Egui will not panic if we try to use a font that is still loading.
                let mut egui_fonts = egui::FontDefinitions::default();
                for font_name in game.ui_theme.font_families.keys() {
                    let font_family = egui::FontFamily::Name(font_name.clone().into());
                    egui_fonts.families.insert(font_family, vec![]);
                }
                egui_ctx.ctx_mut().set_fonts(egui_fonts.clone());
                commands.insert_resource(egui_fonts);

                // Transition to the main menu when we are done
                commands.insert_resource(NextState(GameState::MainMenu));
            }

            // Set the locale resource
            let translations = &game.translations;
            commands.insert_resource(
                Locale::new(translations.detected_locale.clone())
                    .with_default(translations.default_locale.clone()),
            );

            // Spawn the camera
            let mut camera_bundle = OrthographicCameraBundle::new_2d();
            // camera_bundle.orthographic_projection.depth_calculation = DepthCalculation::Distance;
            camera_bundle.orthographic_projection.scaling_mode = ScalingMode::FixedVertical;
            camera_bundle.orthographic_projection.scale = game.camera_height as f32 / 2.0;
            commands
                .spawn_bundle(camera_bundle)
                .insert(ParallaxCameraComponent)
                // We also add another input manager bundle for `MenuAction`s
                .insert_bundle(InputManagerBundle {
                    input_map: menu_input_map(),
                    ..default()
                });

            // Helper to load border images
            let mut load_border_image = |border: &mut BorderImageMeta| {
                border.egui_texture = egui_ctx.add_image(border.handle.clone_weak());
            };

            // Add Border images to egui context
            load_border_image(&mut game.ui_theme.hud.portrait_frame);
            load_border_image(&mut game.ui_theme.panel.border);
            load_border_image(&mut game.ui_theme.hud.lifebar.background_image);
            load_border_image(&mut game.ui_theme.hud.lifebar.progress_image);
            for button in game.ui_theme.button_styles.values_mut() {
                load_border_image(&mut button.borders.default);
                if let Some(border) = &mut button.borders.clicked {
                    load_border_image(border);
                }
                if let Some(border) = &mut button.borders.focused {
                    load_border_image(border);
                }
            }

            // Insert the game resource
            commands.insert_resource(game.clone());
            commands.insert_resource(game.start_level.clone());

        // If the game asset isn't loaded yet
        } else {
            trace!("Awaiting game load")
        }
    }

    // Run checks to see if we should skip running the system
    fn should_skip_run(&mut self, is_hot_reload: bool) -> bool {
        // If this is a hot reload run, check for modified asset events
        if is_hot_reload {
            let mut has_update = false;
            for (event, event_id) in self.events.iter_with_id() {
                if let AssetEvent::Modified { .. } = event {
                    // We may need to skip an asset update event
                    if *self.skip_next_asset_update_event {
                        *self.skip_next_asset_update_event = false;
                    } else {
                        debug!(%event_id, "Game updated");
                        has_update = true;
                    }
                }
            }

            // If there was no update, skip execution
            if !has_update {
                return true;
            }
        }

        false
    }
}

fn menu_input_map() -> InputMap<MenuAction> {
    InputMap::default()
        // Up
        .insert(KeyCode::Up, MenuAction::Up)
        .insert(GamepadButtonType::DPadUp, MenuAction::Up)
        .insert(
            SingleAxis {
                axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickY),
                positive_low: 0.5,
                negative_low: -1.0,
                value: None,
            },
            MenuAction::Up,
        )
        // Left
        .insert(KeyCode::Left, MenuAction::Left)
        .insert(GamepadButtonType::DPadLeft, MenuAction::Left)
        .insert(
            SingleAxis {
                axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickX),
                positive_low: 1.0,
                negative_low: -0.5,
                value: None,
            },
            MenuAction::Left,
        )
        // Down
        .insert(KeyCode::Down, MenuAction::Down)
        .insert(GamepadButtonType::DPadDown, MenuAction::Down)
        .insert(
            SingleAxis {
                axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickY),
                positive_low: 1.0,
                negative_low: -0.5,
                value: None,
            },
            MenuAction::Down,
        )
        // Right
        .insert(KeyCode::Right, MenuAction::Right)
        .insert(GamepadButtonType::DPadRight, MenuAction::Right)
        .insert(
            SingleAxis {
                axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickX),
                positive_low: 0.5,
                negative_low: -1.0,
                value: None,
            },
            MenuAction::Right,
        )
        // Confirm
        .insert(KeyCode::Return, MenuAction::Confirm)
        .insert(GamepadButtonType::South, MenuAction::Confirm)
        .insert(GamepadButtonType::Start, MenuAction::Confirm)
        // Back
        .insert(KeyCode::Escape, MenuAction::Back)
        .insert(GamepadButtonType::East, MenuAction::Back)
        // Toggle Fullscreen
        .insert(KeyCode::F11, MenuAction::ToggleFullscreen)
        .insert(GamepadButtonType::Mode, MenuAction::ToggleFullscreen)
        // Pause
        .insert(KeyCode::Escape, MenuAction::Pause)
        .insert(GamepadButtonType::Start, MenuAction::Pause)
        .build()
}

/// System to run the initial game load
fn load_game(loader: GameLoader) {
    loader.load(false);
}

/// System to check for asset changes and hot reload the game
fn hot_reload_game(loader: GameLoader) {
    loader.load(true);
}

/// Loads a level and transitions to [`GameState::InGame`]
///
/// A [`Handle<Level>`] resource must be inserted before running this system, to indicate which
/// level to load.
fn load_level(
    level_handle: Res<Handle<LevelMeta>>,
    mut commands: Commands,
    assets: Res<Assets<LevelMeta>>,
    mut parallax: ResMut<ParallaxResource>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
    game: Res<GameMeta>,
    windows: Res<Windows>,
    mut storage: ResMut<Storage>,
    loading_resources: LoadingResources,
) {
    if let Some(level) = assets.get(level_handle.clone_weak()) {
        // Track load progress
        let load_progress = level.load_progress(&loading_resources);
        debug!(
            %load_progress,
            "Loading level assets: {:.2}% ",
            load_progress.as_percent()
        );

        // Wait until assets are loaded to start game
        if load_progress.as_percent() < 1.0 {
            return;
        }

        let window = windows.primary();

        // Setup the parallax background
        *parallax = level.parallax_background.get_resource();
        parallax.window_size = Vec2::new(window.width(), window.height());
        parallax.create_layers(&mut commands, &asset_server, &mut texture_atlases);

        // Set the clear color
        commands.insert_resource(ClearColor(level.background_color()));

        // Spawn the players
        for (i, player) in level.players.iter().enumerate() {
            commands.spawn_bundle(PlayerBundle::new(
                player,
                i,
                &game,
                storage.get(Settings::STORAGE_KEY).as_ref(),
            ));
        }

        // Spawn the enemies
        for enemy in &level.enemies {
            commands.spawn_bundle(EnemyBundle::new(enemy));
        }

        // Spawn the items
        for item_spawn_meta in &level.items {
            let item_commands = commands.spawn_bundle(ItemBundle::new(item_spawn_meta));
            ItemBundle::spawn(item_commands, item_spawn_meta)
        }

        commands.insert_resource(level.clone());
        commands.insert_resource(NextState(GameState::InGame));
    } else {
        trace!("Awaiting level load");
    }
}

/// Hot reloads level asset data
fn hot_reload_level(
    mut commands: Commands,
    mut parallax: ResMut<ParallaxResource>,
    mut events: EventReader<AssetEvent<LevelMeta>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    level_handle: Res<Handle<LevelMeta>>,
    assets: Res<Assets<LevelMeta>>,
    asset_server: Res<AssetServer>,
    windows: Res<Windows>,
) {
    for event in events.iter() {
        if let AssetEvent::Modified { handle } = event {
            let level = assets.get(handle).unwrap();
            if handle == &*level_handle {
                // Update the level background
                let window = windows.primary();
                parallax.despawn_layers(&mut commands);
                *parallax = level.parallax_background.get_resource();
                parallax.window_size = Vec2::new(window.width(), window.height());
                parallax.create_layers(&mut commands, &asset_server, &mut texture_atlases);

                commands.insert_resource(ClearColor(level.background_color()));
            }
        }
    }
}

fn load_items(
    mut commands: Commands,
    item_spawns: Query<(Entity, &Transform, &Handle<ItemMeta>), Without<Sprite>>,
    item_assets: Res<Assets<ItemMeta>>,
) {
    for (entity, transform, item_handle) in item_spawns.iter() {
        if let Some(item_meta) = item_assets.get(item_handle) {
            commands.entity(entity).insert_bundle(SpriteBundle {
                texture: item_meta.image.image_handle.clone(),
                transform: *transform,
                ..default()
            });
        }
    }
}

/// Load all fighters that have their handles spawned.
///
/// Fighters are spawned as "stubs" that only contain a transform, a marker component, and a
/// [`Handle<Fighter>`]. This system takes those stubs, populates the rest of their components once
/// the figher asset has been loaded.
fn load_fighters(
    mut commands: Commands,
    // All fighters that haven't been fully loaded yet
    fighters: Query<
        (
            Entity,
            &Transform,
            &Handle<FighterMeta>,
            Option<&Player>,
            Option<&Enemy>,
        ),
        Without<Stats>,
    >,
    fighter_assets: Res<Assets<FighterMeta>>,
) {
    for (entity, transform, fighter_handle, player, enemy) in fighters.iter() {
        if let Some(fighter) = fighter_assets.get(fighter_handle) {
            let body_layers = if player.is_some() {
                BodyLayers::PLAYER
            } else if enemy.is_some() {
                BodyLayers::ENEMY
            } else {
                unreachable!();
            };

            commands
                .entity(entity)
                .insert(Name::new(fighter.name.clone()))
                .insert_bundle(AnimatedSpriteSheetBundle {
                    sprite_sheet: SpriteSheetBundle {
                        sprite: TextureAtlasSprite::new(0),
                        texture_atlas: fighter
                            .spritesheet
                            .atlas_handle
                            .choose(&mut rand::thread_rng())
                            .unwrap()
                            .clone(),
                        transform: *transform,
                        ..Default::default()
                    },
                    animation: Animation::new(
                        fighter.spritesheet.animation_fps,
                        fighter.spritesheet.animations.clone(),
                    ),
                })
                .insert_bundle(CharacterBundle {
                    stats: fighter.stats.clone(),
                    ..default()
                })
                .insert_bundle(PhysicsBundle {
                    collision_groups: CollisionGroups::new(body_layers, BodyLayers::ALL),
                    ..default()
                });
        }
    }
}

/// Hot reload fighter data when fighter assets are updated.
fn hot_reload_fighters(
    mut fighters: Query<(
        &Handle<FighterMeta>,
        &mut Name,
        &mut Handle<TextureAtlas>,
        &mut Animation,
        &mut Stats,
    )>,
    mut events: EventReader<AssetEvent<FighterMeta>>,
    assets: Res<Assets<FighterMeta>>,
) {
    for event in events.iter() {
        if let AssetEvent::Modified { handle } = event {
            for (fighter_handle, mut name, mut atlas_handle, mut animation, mut stats) in
                fighters.iter_mut()
            {
                if fighter_handle == handle {
                    let fighter = assets.get(fighter_handle).unwrap();

                    *name = Name::new(fighter.name.clone());
                    *atlas_handle = fighter
                        .spritesheet
                        .atlas_handle
                        .choose(&mut rand::thread_rng())
                        .unwrap()
                        .clone();
                    *animation = Animation::new(
                        fighter.spritesheet.animation_fps,
                        fighter.spritesheet.animations.clone(),
                    );
                    *stats = fighter.stats.clone();
                }
            }
        }
    }
}
