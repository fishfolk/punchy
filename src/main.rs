#![allow(clippy::type_complexity)]
#![allow(clippy::forget_non_drop)]

use bevy::{
    asset::{AssetServerSettings, AssetStage},
    ecs::bundle::Bundle,
    prelude::*,
    render::camera::ScalingMode,
};
use bevy_parallax::{ParallaxCameraComponent, ParallaxPlugin, ParallaxResource};
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;
use rand::Rng;
use structopt::StructOpt;

#[cfg(feature = "debug")]
use bevy_inspector_egui::{RegisterInspectable, WorldInspectorPlugin};
#[cfg(feature = "debug")]
use bevy_inspector_egui_rapier::InspectableRapierPlugin;

#[cfg(feature = "schedule_graph")]
use bevy::log::LogPlugin;

mod animation;
mod assets;
mod attack;
mod camera;
mod collisions;
mod config;
mod consts;
mod item;
mod metadata;
mod movement;
mod platform;
mod state;
mod ui;
mod y_sort;

use animation::*;
use attack::{Attack, AttackPlugin};
use camera::*;
use collisions::*;
use item::{spawn_throwable_items, ThrowItemEvent};
use metadata::{Fighter, Game, Level};
use movement::*;
use serde::Deserialize;
use state::{State, StatePlugin};
use ui::UIPlugin;
use y_sort::*;

use crate::config::EngineConfig;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Enemy;

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
    HotReload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
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

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    facing: Facing,
}
impl Default for PlayerBundle {
    fn default() -> Self {
        PlayerBundle {
            player: Player,
            facing: Facing::Right,
        }
    }
}

#[derive(Bundle)]
struct EnemyBundle {
    enemy: Enemy,
    facing: Facing,
}
impl Default for EnemyBundle {
    fn default() -> Self {
        EnemyBundle {
            enemy: Enemy,
            facing: Facing::Left,
        }
    }
}

fn main() {
    let engine_config = EngineConfig::from_args();

    let mut app = App::new();

    let mut asset_server_settings = AssetServerSettings {
        watch_for_changes: engine_config.hot_reload,
        ..default()
    };

    if let Some(asset_dir) = &engine_config.asset_dir {
        asset_server_settings.asset_folder = asset_dir.clone();
    }

    #[cfg(feature = "schedule_graph")]
    app.add_plugins_with(DefaultPlugins, |plugins| {
        plugins.disable::<bevy::log::LogPlugin>()
    });
    #[cfg(not(feature = "schedule_graph"))]
    app.add_plugins(DefaultPlugins);
    app.insert_resource(engine_config.clone())
        .insert_resource(asset_server_settings)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            title: "Fish Fight Punchy".to_string(),
            scale_factor_override: Some(1.0),
            ..Default::default()
        })
        .add_event::<ThrowItemEvent>()
        .add_plugin(platform::PlatformPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(AttackPlugin)
        .add_plugin(AnimationPlugin)
        .add_plugin(StatePlugin)
        .add_plugin(ParallaxPlugin)
        .add_plugin(UIPlugin)
        .insert_resource(ParallaxResource::default())
        .add_startup_system(spawn_camera)
        .add_loopless_state(GameState::LoadingGame)
        .add_system(load_game.run_in_state(GameState::LoadingGame))
        .add_system(load_level.run_in_state(GameState::LoadingLevel))
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(load_fighters)
                .with_system(spawn_throwable_items)
                .with_system(player_controller)
                .with_system(player_attack)
                .with_system(helper_camera_controller)
                .with_system(y_sort)
                .with_system(player_attack_enemy_collision)
                .with_system(player_enemy_collision)
                .with_system(kill_entities)
                .with_system(knockback_system)
                .with_system(move_direction_system)
                .with_system(move_in_arc_system)
                .with_system(throw_item_system)
                .with_system(item_attacks_enemy_collision)
                .with_system(rotate_system)
                .with_system(set_target_near_player)
                .with_system(move_to_target)
                .with_system(pause)
                .into(),
        )
        .add_system(unpause.run_in_state(GameState::Paused))
        .add_system_to_stage(
            CoreStage::PostUpdate,
            camera_follow_player.run_in_state(GameState::InGame),
        )
        .add_system_to_stage(CoreStage::Last, despawn_entities);

    if engine_config.hot_reload {
        app.add_stage_after(
            AssetStage::LoadAssets,
            GameStage::HotReload,
            SystemStage::parallel(),
        )
        .add_system_set_to_stage(
            GameStage::HotReload,
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(hot_reload_level)
                .with_system(hot_reload_fighters)
                .into(),
        );
    }

    #[cfg(feature = "debug")]
    app.add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(InspectableRapierPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .register_inspectable::<Stats>()
        .register_inspectable::<State>()
        .register_inspectable::<MoveInDirection>()
        .register_inspectable::<MoveInArc>()
        .register_inspectable::<Rotate>()
        .register_inspectable::<Attack>()
        .register_inspectable::<YSort>()
        .register_inspectable::<Facing>()
        .register_inspectable::<Panning>();

    assets::register(&mut app);

    debug!(?engine_config, "Starting game");

    // Insert the game handle
    let asset_server = app.world.get_resource::<AssetServer>().unwrap();
    let game_asset = engine_config.game_asset;
    let handle: Handle<Game> = asset_server.load(&game_asset);
    app.world.insert_resource(handle);

    #[cfg(feature = "schedule_graph")]
    bevy_mod_debugdump::print_schedule(&mut app);

    app.run();
}

fn spawn_camera(mut commands: Commands) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    // camera_bundle.orthographic_projection.depth_calculation = DepthCalculation::Distance;
    camera_bundle.orthographic_projection.scaling_mode = ScalingMode::FixedVertical;
    camera_bundle.orthographic_projection.scale = 16. * 14.;
    commands
        .spawn_bundle(camera_bundle)
        .insert(Panning {
            offset: Vec2::new(0., -consts::GROUND_Y),
        })
        .insert(ParallaxCameraComponent);
}

fn load_game(
    mut commands: Commands,
    game_handle: Res<Handle<Game>>,
    mut assets: ResMut<Assets<Game>>,
) {
    if let Some(game) = assets.remove(game_handle.clone_weak()) {
        debug!("Loaded game");
        commands.insert_resource(game.start_level.clone());
        commands.insert_resource(game);
        commands.insert_resource(NextState(GameState::LoadingLevel));
    } else {
        trace!("Awaiting game load")
    }
}

fn load_level(
    level_handle: Res<Handle<Level>>,
    mut commands: Commands,
    assets: Res<Assets<Level>>,
    mut parallax: ResMut<ParallaxResource>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
    windows: Res<Windows>,
) {
    if let Some(level) = assets.get(level_handle.clone_weak()) {
        debug!("Loaded level");
        let window = windows.primary();

        // Setup the parallax background
        *parallax = level.meta.parallax_background.get_resource();
        parallax.window_size = Vec2::new(window.width(), window.height());
        parallax.create_layers(&mut commands, &asset_server, &mut texture_atlases);

        // Set the clear color
        commands.insert_resource(ClearColor(level.meta.background_color()));

        // Spawn the player
        let ground_offset = Vec3::new(0.0, consts::GROUND_Y, 0.0);
        let player_pos = level.meta.player_spawn.location + ground_offset;
        commands
            .spawn_bundle(TransformBundle::from_transform(
                Transform::from_translation(player_pos),
            ))
            .insert(level.player_fighter_handle.clone())
            .insert_bundle(PlayerBundle::default());

        // Spawn the enemies
        for (enemy, enemy_handle) in level
            .meta
            .enemies
            .iter()
            .zip(level.enemy_fighter_handles.iter())
        {
            let enemy_pos = enemy.location + ground_offset;
            commands
                .spawn_bundle(TransformBundle::from_transform(
                    Transform::from_translation(enemy_pos),
                ))
                .insert(enemy_handle.clone())
                .insert_bundle(EnemyBundle::default());
        }

        commands.insert_resource(level.clone());
        commands.insert_resource(NextState(GameState::InGame));
    } else {
        trace!("Awaiting level load");
    }
}

fn hot_reload_level(
    mut commands: Commands,
    mut parallax: ResMut<ParallaxResource>,
    mut events: EventReader<AssetEvent<Level>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    level_handle: Res<Handle<Level>>,
    assets: Res<Assets<Level>>,
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
                *parallax = level.meta.parallax_background.get_resource();
                parallax.window_size = Vec2::new(window.width(), window.height());
                parallax.create_layers(&mut commands, &asset_server, &mut texture_atlases);

                commands.insert_resource(ClearColor(level.meta.background_color()));
            }
        }
    }
}

/// Load all fighters that have their handles spawned
fn load_fighters(
    mut commands: Commands,
    // All fighters that haven't been fully loaded yet
    fighters: Query<
        (
            Entity,
            &Transform,
            &Handle<Fighter>,
            Option<&Player>,
            Option<&Enemy>,
        ),
        Without<Stats>,
    >,
    fighter_assets: Res<Assets<Fighter>>,
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
                .insert(Name::new(fighter.meta.name.clone()))
                .insert_bundle(AnimatedSpriteSheetBundle {
                    sprite_sheet: SpriteSheetBundle {
                        sprite: TextureAtlasSprite::new(0),
                        texture_atlas: fighter.atlas_handle.clone(),
                        transform: *transform,
                        ..Default::default()
                    },
                    animation: Animation::new(
                        fighter.meta.spritesheet.animation_fps,
                        fighter.meta.spritesheet.animations.clone(),
                    ),
                })
                .insert_bundle(CharacterBundle {
                    stats: fighter.meta.stats.clone(),
                    ..default()
                })
                .insert_bundle(PhysicsBundle {
                    collision_groups: CollisionGroups::new(body_layers, BodyLayers::ALL),
                    ..default()
                });
        }
    }
}

fn hot_reload_fighters(
    mut fighters: Query<(
        &Handle<Fighter>,
        &mut Name,
        &mut Handle<TextureAtlas>,
        &mut Animation,
        &mut Stats,
    )>,
    mut events: EventReader<AssetEvent<Fighter>>,
    assets: Res<Assets<Fighter>>,
) {
    for event in events.iter() {
        if let AssetEvent::Modified { handle } = event {
            for (fighter_handle, mut name, mut atlas_handle, mut animation, mut stats) in
                fighters.iter_mut()
            {
                if fighter_handle == handle {
                    let fighter = assets.get(fighter_handle).unwrap();

                    *name = Name::new(fighter.meta.name.clone());
                    *atlas_handle = fighter.atlas_handle.clone();
                    *animation = Animation::new(
                        fighter.meta.spritesheet.animation_fps,
                        fighter.meta.spritesheet.animations.clone(),
                    );
                    *stats = fighter.meta.stats.clone();
                }
            }
        }
    }
}

fn pause(keyboard: Res<Input<KeyCode>>, mut commands: Commands) {
    if keyboard.just_pressed(KeyCode::P) {
        commands.insert_resource(NextState(GameState::Paused));
    }
}

fn unpause(keyboard: Res<Input<KeyCode>>, mut commands: Commands) {
    if keyboard.just_pressed(KeyCode::P) {
        commands.insert_resource(NextState(GameState::InGame));
    }
}
fn player_attack(
    mut query: Query<(&mut State, &mut Transform, &Animation, &Facing), With<Player>>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    if let Ok((mut state, mut transform, animation, facing)) = query.get_single_mut() {
        if *state != State::Attacking {
            if keyboard.just_pressed(KeyCode::Space) {
                state.set(State::Attacking);
            }
        } else if animation.is_finished() {
            state.set(State::Idle);
        } else {
            //TODO: Fix hacky way to get a forward jump
            if animation.current_frame < 3 {
                if facing.is_left() {
                    transform.translation.x -= 200. * time.delta_seconds();
                } else {
                    transform.translation.x += 200. * time.delta_seconds();
                }
            }

            if animation.current_frame < 1 {
                transform.translation.y += 180. * time.delta_seconds();
            } else if animation.current_frame < 3 {
                transform.translation.y -= 90. * time.delta_seconds();
            }
        }
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

fn set_target_near_player(
    mut commands: Commands,
    query: Query<(Entity, &Transform), (With<Enemy>, Without<Target>)>,
    player_query: Query<&Transform, With<Player>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let mut rng = rand::thread_rng();

        for (entity, transform) in query.iter() {
            if transform
                .translation
                .truncate()
                .distance(player_transform.translation.truncate())
                >= 100.0
            {
                let x_offset = rng.gen_range(-100.0..100.);
                let y_offset = rng.gen_range(-100.0..100.);
                commands.entity(entity).insert(Target {
                    position: player_transform.translation.truncate()
                        + Vec2::new(x_offset, y_offset),
                });
            }
        }
    }
}
