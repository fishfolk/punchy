#![allow(clippy::type_complexity)]
#![allow(clippy::forget_non_drop)]
#![allow(clippy::too_many_arguments)]

use bevy::{
    asset::AssetServerSettings, log::LogSettings, prelude::*, render::texture::ImageSettings,
};
use bevy_kira_audio::AudioApp;
use bevy_parallax::{ParallaxPlugin, ParallaxResource};
use bevy_rapier2d::prelude::*;
use fighter::Stats;
use input::MenuAction;
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::*;
use player::*;

#[cfg(feature = "debug")]
use bevy_inspector_egui::{RegisterInspectable, WorldInspectorPlugin};
#[cfg(feature = "debug")]
use bevy_inspector_egui_rapier::InspectableRapierPlugin;

#[cfg(feature = "schedule_graph")]
use bevy::log::LogPlugin;

mod animation;
mod assets;
mod attack;
mod audio;
mod camera;
mod collision;
mod commands;
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
mod ui;
mod utils;
mod y_sort;

use animation::*;
use attack::AttackPlugin;
use audio::*;
use camera::*;
use metadata::GameMeta;
use ui::UIPlugin;
use utils::ResetController;
use y_sort::*;

use crate::{
    damage::DamagePlugin,
    fighter_state::FighterStatePlugin,
    input::PlayerAction,
    item::pick_items,
    lifetime::LifetimePlugin,
    movement::{LeftMovementBoundary, MovementPlugin},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, StageLabel)]
enum GameStage {
    Animation,
    HotReload,
}

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
        .add_stage_after(
            CoreStage::Update,
            GameStage::Animation,
            SystemStage::parallel(),
        )
        .add_loopless_state(GameState::LoadingStorage)
        .add_plugin(platform::PlatformPlugin)
        .add_plugin(localization::LocalizationPlugin)
        .add_plugin(loading::LoadingPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
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
        .add_audio_channel::<MusicChannel>()
        .add_audio_channel::<EffectsChannel>()
        .insert_resource(ParallaxResource::default())
        .insert_resource(LeftMovementBoundary::default())
        .add_system(platform::load_storage.run_in_state(GameState::LoadingStorage))
        .add_startup_system(set_audio_channels_volume)
        .add_enter_system(GameState::InGame, play_level_music)
        .add_exit_system(GameState::InGame, stop_level_music)
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(pick_items)
                .with_system(use_health_item)
                .with_system(y_sort)
                // .with_system(attack_fighter_collision)
                // .with_system(kill_entities)
                .with_system(pause)
                .into(),
        )
        .add_system(unpause.run_in_state(GameState::Paused))
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(camera_follow_player)
                .with_system(game_over_on_players_death)
                .into(),
        );

    // Add debug plugins
    #[cfg(feature = "debug")]
    app.add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(InspectableRapierPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .register_inspectable::<Stats>()
        .register_inspectable::<crate::movement::LinearVelocity>()
        .register_inspectable::<crate::movement::AngularVelocity>()
        // .register_inspectable::<MoveInArc>()
        .register_inspectable::<attack::Attack>()
        .register_inspectable::<YSort>()
        .register_inspectable::<Facing>();

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

/// Transition game to pause state
fn pause(mut commands: Commands, input: Query<&ActionState<MenuAction>>) {
    let input = input.single();
    if input.just_pressed(MenuAction::Pause) {
        commands.insert_resource(NextState(GameState::Paused));
    }
}

// Transition game out of paused state
fn unpause(mut commands: Commands, input: Query<&ActionState<MenuAction>>) {
    let input = input.single();
    if input.just_pressed(MenuAction::Pause) {
        commands.insert_resource(NextState(GameState::InGame));
    }
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
