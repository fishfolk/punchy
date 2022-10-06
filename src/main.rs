#![allow(clippy::type_complexity)]
#![allow(clippy::forget_non_drop)]
#![allow(clippy::too_many_arguments)]

use bevy::{
    asset::AssetServerSettings, log::LogSettings, prelude::*, render::texture::ImageSettings,
};
use bevy_parallax::{ParallaxPlugin, ParallaxResource};
use bevy_rapier2d::prelude::*;
use fighter::Stats;
use input::MenuAction;
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::*;
use player::*;

use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_inspector_egui_rapier::InspectableRapierPlugin;

#[cfg(feature = "schedule_graph")]
use bevy::log::LogPlugin;

mod animation;
mod assets;
mod attack;
mod audio;
mod camera;
mod collision;
mod config;
mod consts;
mod damage;
mod enemy;
mod enemy_ai;
mod fighter;
mod fighter_state;
mod input;
mod item;
mod lifetime;
mod loading;
mod localization;
mod metadata;
mod movement;
mod platform;
mod player;
mod scripting;
mod ui;
mod utils;

use animation::*;
use attack::AttackPlugin;
use audio::*;
use camera::*;
use metadata::GameMeta;
use ui::UIPlugin;
use utils::ResetController;

use crate::{
    damage::DamagePlugin, fighter::FighterPlugin, fighter_state::FighterStatePlugin,
    input::PlayerAction, item::ItemPlugin, lifetime::LifetimePlugin, loading::LoadingPlugin,
    localization::LocalizationPlugin, movement::MovementPlugin, platform::PlatformPlugin,
    scripting::ScriptingPlugin, ui::debug_tools::YSortDebugPlugin,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    LoadingStorage,
    LoadingGame,
    MainMenu,
    LoadingLevel,
    InGame,
    Paused,
    //Editor,
}

fn main() {
    // Load engine config. This will parse CLI arguments or web query string so we want to do it
    // before we create the app to make sure everything is in order.
    let engine_config = &*config::ENGINE_CONFIG;

    let mut app = App::new();
    app.insert_resource(WindowDescriptor {
        title: "Fish Folk Punchy".to_string(),
        scale_factor_override: Some(1.0),
        ..default()
    })
    .insert_resource(ImageSettings::default_nearest());

    // Configure log level
    app.insert_resource(LogSettings {
        filter: engine_config.log_level.clone(),
        ..default()
    });

    // Configure asset server
    let mut asset_server_settings = AssetServerSettings {
        watch_for_changes: engine_config.hot_reload,
        ..default()
    };
    if let Some(asset_dir) = &engine_config.asset_dir {
        asset_server_settings.asset_folder = asset_dir.clone();
    }
    app.insert_resource(asset_server_settings);
    // Add default plugins
    #[cfg(feature = "schedule_graph")]
    app.add_plugins_with(DefaultPlugins, |plugins| {
        plugins.disable::<bevy::log::LogPlugin>()
    });
    #[cfg(not(feature = "schedule_graph"))]
    app.add_plugins(DefaultPlugins);

    // Add other systems and resources
    app.insert_resource(ClearColor(Color::BLACK))
        .add_loopless_state(GameState::LoadingStorage)
        .add_plugin(ScriptingPlugin)
        .add_plugin(PlatformPlugin)
        .add_plugin(LocalizationPlugin)
        .add_plugin(LoadingPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(InputManagerPlugin::<PlayerAction>::default())
        .add_plugin(InputManagerPlugin::<MenuAction>::default())
        .add_plugin(AttackPlugin)
        .add_plugin(AnimationPlugin)
        .add_plugin(ParallaxPlugin)
        .add_plugin(UIPlugin)
        .add_plugin(FighterStatePlugin)
        .add_plugin(MovementPlugin)
        .add_plugin(AudioPlugin)
        .add_plugin(DamagePlugin)
        .add_plugin(LifetimePlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(ItemPlugin)
        .add_plugin(FighterPlugin)
        .insert_resource(ParallaxResource::default())
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(game_over_on_players_death)
                .into(),
        )
        .add_system_to_stage(
            CoreStage::PostUpdate,
            main_menu_sounds
                .run_if_resource_exists::<GameMeta>()
                .before(bevy_egui::EguiSystem::ProcessOutput),
        );

    // Register reflect types that don't come from plugins
    app.register_type::<Stats>();

    // Add debug plugins if enabled
    if engine_config.debug_tools {
        app.insert_resource(DebugRenderContext {
            enabled: false,
            ..default()
        })
        .add_plugin(YSortDebugPlugin)
        .add_plugin(InspectableRapierPlugin)
        .insert_resource(WorldInspectorParams {
            enabled: false,
            ..default()
        })
        .add_plugin(WorldInspectorPlugin::new());
    }

    // Register assets and loaders
    assets::register(&mut app);

    debug!(?engine_config, "Starting game");

    // Get the game handle
    let asset_server = app.world.get_resource::<AssetServer>().unwrap();
    let game_asset = &engine_config.game_asset;
    let game_handle: Handle<GameMeta> = asset_server.load(game_asset);

    // Insert game handle resource
    app.world.insert_resource(game_handle);

    // Print the graphviz schedule graph
    #[cfg(feature = "schedule_graph")]
    bevy_mod_debugdump::print_schedule(&mut app);

    app.run();
}

/// Transition back to main menu and reset world when all players have died
fn game_over_on_players_death(
    mut commands: Commands,
    query: Query<(), With<Player>>,
    reset_controller: ResetController,
) {
    if query.is_empty() {
        commands.insert_resource(NextState(GameState::MainMenu));

        reset_controller.reset_world();
    }
}
