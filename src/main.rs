#![allow(clippy::type_complexity)]
#![allow(clippy::forget_non_drop)]
#![allow(clippy::too_many_arguments)]

use bevy::{
    asset::{AssetServerSettings, AssetStage},
    ecs::bundle::Bundle,
    log::LogSettings,
    prelude::*,
};
use bevy_kira_audio::{AudioApp, AudioPlugin};
use bevy_parallax::{ParallaxPlugin, ParallaxResource};
use bevy_rapier2d::prelude::*;
use enemy::*;
use input::MenuAction;
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::*;
use platform::Storage;
use player::*;
use rand::{seq::SliceRandom, Rng};

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
mod collisions;
mod config;
mod consts;
mod enemy;
mod game_init;
mod input;
mod item;
mod localization;
mod metadata;
mod movement;
mod platform;
mod player;
mod state;
mod ui;
mod y_sort;

use animation::*;
use attack::AttackPlugin;
use audio::*;
use camera::*;
use collisions::*;
use metadata::{FighterMeta, GameMeta, ItemMeta, LevelMeta};
use movement::*;
use serde::Deserialize;
use state::{State, StatePlugin};
use ui::UIPlugin;
use y_sort::*;

use crate::{config::EngineConfig, input::PlayerAction, item::ItemSpawnBundle, metadata::Settings};

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Stats {
    pub health: i32,
    pub damage: i32,
    pub movement_speed: f32,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            health: 100,
            damage: 35,
            movement_speed: 150.,
        }
    }
}

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

#[derive(Component)]
pub struct DespawnMarker;

#[derive(Bundle, Default)]
struct CharacterBundle {
    state: State,
    stats: Stats,
    ysort: YSort,
}

#[derive(Bundle)]
struct AnimatedSpriteSheetBundle {
    #[bundle]
    sprite_sheet: SpriteSheetBundle,
    animation: Animation,
}

#[derive(Bundle)]
struct PhysicsBundle {
    collider: Collider,
    sensor: Sensor,
    active_events: ActiveEvents,
    active_collision_types: ActiveCollisionTypes,
    collision_groups: CollisionGroups,
}
impl Default for PhysicsBundle {
    fn default() -> Self {
        PhysicsBundle {
            collider: (Collider::cuboid(
                consts::PLAYER_SPRITE_WIDTH / 8.,
                consts::PLAYER_HITBOX_HEIGHT / 8.,
            )),
            sensor: Sensor(true),
            active_events: ActiveEvents::COLLISION_EVENTS,
            active_collision_types: ActiveCollisionTypes::default()
                | ActiveCollisionTypes::STATIC_STATIC,
            collision_groups: CollisionGroups::default(),
        }
    }
}

pub struct ArrivedEvent(Entity);

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    let engine_config = {
        use structopt::StructOpt;
        EngineConfig::from_args()
    };

    #[cfg(target_arch = "wasm32")]
    let engine_config = EngineConfig::from_web_params();

    let mut app = App::new();
    app.insert_resource(WindowDescriptor {
        title: "Fish Fight Punchy".to_string(),
        scale_factor_override: Some(1.0),
        ..default()
    });

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
    app.insert_resource(engine_config.clone())
        .insert_resource(ClearColor(Color::BLACK))
        .add_stage_after(
            CoreStage::Update,
            GameStage::Animation,
            SystemStage::parallel(),
        )
        .add_event::<ArrivedEvent>()
        .add_loopless_state(GameState::LoadingStorage)
        .add_plugin(platform::PlatformPlugin)
        .add_plugin(localization::LocalizationPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(InputManagerPlugin::<PlayerAction>::default())
        .add_plugin(InputManagerPlugin::<MenuAction>::default())
        .add_plugin(AttackPlugin)
        .add_plugin(AnimationPlugin)
        .add_plugin(AudioPlugin)
        .add_plugin(StatePlugin)
        .add_plugin(ParallaxPlugin)
        .add_plugin(UIPlugin)
        .add_audio_channel::<MusicChannel>()
        .add_audio_channel::<EffectsChannel>()
        .insert_resource(ParallaxResource::default())
        .insert_resource(LeftMovementBoundary::default())
        .add_system(platform::load_storage.run_in_state(GameState::LoadingStorage))
        .add_startup_system(set_audio_channels_volume)
        .add_enter_system(GameState::InGame, play_level_music)
        .add_exit_system(GameState::InGame, stop_level_music)
        .add_exit_system(GameState::InGame, clean_in_game_data)
        .add_system(game_init::load_game.run_in_state(GameState::LoadingGame))
        .add_system(load_level.run_in_state(GameState::LoadingLevel))
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(load_fighters)
                .with_system(load_items)
                .with_system(player_controller)
                .with_system(y_sort)
                .with_system(attack_fighter_collision)
                .with_system(kill_entities)
                .with_system(knockback_system)
                .with_system(move_direction_system)
                .with_system(pause)
                .into(),
        )
        .add_system(
            set_target_near_player
                .run_in_state(GameState::InGame)
                .label("set_target_near_player"),
        )
        .add_system(
            move_to_target
                .run_in_state(GameState::InGame)
                .after("set_target_near_player")
                .label("move_to_target"),
        )
        .add_system(unpause.run_in_state(GameState::Paused))
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(move_in_arc_system)
                .with_system(rotate_system)
                .with_system(camera_follow_player)
                .with_system(update_left_movement_boundary)
                .with_system(fighter_sound_effect)
                .with_system(game_over_on_players_death)
                .into(),
        )
        .add_system_to_stage(CoreStage::Last, despawn_entities);

    // Add debug plugins
    #[cfg(feature = "debug")]
    app.add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(InspectableRapierPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .register_inspectable::<Stats>()
        .register_inspectable::<State>()
        .register_inspectable::<MoveInDirection>()
        .register_inspectable::<MoveInArc>()
        .register_inspectable::<Rotate>()
        .register_inspectable::<attack::Attack>()
        .register_inspectable::<YSort>()
        .register_inspectable::<Facing>();

    // Register assets and loaders
    assets::register(&mut app);

    debug!(?engine_config, "Starting game");

    // Get the game handle
    let asset_server = app.world.get_resource::<AssetServer>().unwrap();
    let game_asset = engine_config.game_asset;
    let game_handle: Handle<GameMeta> = asset_server.load(&game_asset);

    // Configure hot reload
    if engine_config.hot_reload {
        app.add_stage_after(
            AssetStage::LoadAssets,
            GameStage::HotReload,
            SystemStage::parallel(),
        )
        .add_system_to_stage(GameStage::HotReload, game_init::hot_reload_game)
        .add_system_set_to_stage(
            GameStage::HotReload,
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(hot_reload_level)
                .with_system(hot_reload_fighters)
                .into(),
        );
    }

    // Insert game handle resource
    app.world.insert_resource(game_handle);

    // Print the graphviz schedule graph
    #[cfg(feature = "schedule_graph")]
    bevy_mod_debugdump::print_schedule(&mut app);

    app.run();
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
) {
    if let Some(level) = assets.get(level_handle.clone_weak()) {
        debug!("Loaded level");
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
            commands.spawn_bundle(ItemSpawnBundle::new(item_spawn_meta));
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
                        texture_atlas: fighter.spritesheet.atlas_handle.clone(),
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
                    *atlas_handle = fighter.spritesheet.atlas_handle.clone();
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

fn kill_entities(
    mut commands: Commands,
    mut query: Query<(Entity, &Stats, &Animation, &mut State)>,
) {
    for (entity, stats, animation, mut state) in query.iter_mut() {
        if stats.health <= 0 {
            state.set(State::Dying);
        }

        if *state == State::Dying && animation.is_finished() {
            commands.entity(entity).insert(DespawnMarker);
            // commands.entity(entity).despawn_recursive();
        }
    }
}

fn despawn_entities(mut commands: Commands, query: Query<Entity, With<DespawnMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn game_over_on_players_death(mut commands: Commands, query: Query<(), With<Player>>) {
    if query.is_empty() {
        commands.insert_resource(NextState(GameState::MainMenu));
    }
}

fn clean_in_game_data(mut commands: Commands, query: Query<Entity, Without<Camera>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

//for enemys without current target, pick a new spot near the player as target
fn set_target_near_player(
    mut commands: Commands,
    query: Query<(Entity, &State), (With<Enemy>, Without<Target>)>,
    player_query: Query<&Transform, With<Player>>,
) {
    let mut rng = rand::thread_rng();
    let transforms = player_query.iter().collect::<Vec<_>>();

    for (entity, state) in query.iter() {
        if *state == State::Idle {
            if let Some(player_transform) = transforms.choose(&mut rng) {
                let x_offset = rng.gen_range(-100.0..100.);
                let y_offset = rng.gen_range(-100.0..100.);
                commands.entity(entity).insert(Target {
                    position: Vec2::new(
                        player_transform.translation.x + x_offset,
                        (player_transform.translation.y + y_offset)
                            .clamp(consts::MIN_Y, consts::MAX_Y),
                    ),
                });
            }
        }
    }
}
