use bevy::{ecs::bundle::Bundle, prelude::*, render::camera::ScalingMode, utils::HashMap};
use bevy_parallax::{LayerData, ParallaxCameraComponent, ParallaxPlugin, ParallaxResource};
use bevy_rapier2d::prelude::*;

mod animation;
mod attack;
mod camera;
mod collisions;
mod consts;
mod item;
mod movement;
mod state;
mod y_sort;

use animation::*;
use attack::AttackPlugin;
use camera::*;
use collisions::*;
use item::{spawn_throwable_items, ThrowItemEvent};
use movement::*;
use state::{State, StatePlugin};
use y_sort::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
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
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::rgb(0.494, 0.658, 0.650)))
        .insert_resource(WindowDescriptor {
            title: "Fish Fight Punchy".to_string(),
            scale_factor_override: Some(1.0),
            ..Default::default()
        })
        .add_event::<ThrowItemEvent>()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(AttackPlugin)
        .add_plugin(AnimationPlugin)
        .add_plugin(StatePlugin)
        .insert_resource(ParallaxResource {
            layer_data: vec![
                LayerData {
                    speed: 0.98,
                    path: "beach/background_01.png".to_string(),
                    tile_size: Vec2::new(960.0, 540.0),
                    cols: 1,
                    rows: 1,
                    z: 0.0,
                    scale: 0.9,
                    transition_factor: 0.9,
                },
                LayerData {
                    speed: 0.9,
                    path: "beach/background_02.2.png".to_string(),
                    tile_size: Vec2::new(960.0, 540.0),
                    cols: 1,
                    rows: 1,
                    z: 1.0,
                    scale: 0.9,
                    transition_factor: 0.9,
                },
                LayerData {
                    speed: 0.82,
                    path: "beach/background_03.png".to_string(),
                    tile_size: Vec2::new(960.0, 540.0),
                    cols: 1,
                    rows: 1,
                    z: 2.0,
                    scale: 0.9,
                    transition_factor: 0.9,
                },
                LayerData {
                    speed: 0.74,
                    path: "beach/background_04.2.png".to_string(),
                    tile_size: Vec2::new(960.0, 540.0),
                    cols: 1,
                    rows: 1,
                    z: 3.0,
                    scale: 0.9,
                    transition_factor: 0.9,
                },
                LayerData {
                    speed: 0.,
                    path: "beach/background_05.2.png".to_string(),
                    tile_size: Vec2::new(960.0, 540.0),
                    cols: 1,
                    rows: 1,
                    z: 4.0,
                    scale: 0.9,
                    transition_factor: 0.9,
                },
            ],
            ..Default::default()
        })
        .add_plugin(ParallaxPlugin)
        .add_startup_system(setup)
        .add_system(spawn_throwable_items)
        .add_system(player_controller)
        .add_system_to_stage(CoreStage::PostUpdate, camera_follow_player)
        .add_system(player_attack)
        .add_system(helper_camera_controller)
        .add_system(y_sort)
        .add_system(player_attack_enemy_collision)
        .add_system(player_enemy_collision)
        .add_system(kill_entities)
        .add_system(knockback_system)
        .add_system(move_direction_system)
        .add_system(move_in_arc_system)
        .add_system(throw_item_system)
        .add_system(item_attacks_enemy_collision)
        .add_system(rotate_system)
        .add_system_to_stage(CoreStage::Last, despawn_entities);
    app.run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    // camera_bundle.orthographic_projection.depth_calculation = DepthCalculation::Distance;
    camera_bundle.orthographic_projection.scaling_mode = ScalingMode::FixedVertical;
    camera_bundle.orthographic_projection.scale = 16. * 14.;
    commands
        .spawn_bundle(camera_bundle)
        .insert(ParallaxCameraComponent);

    let texture_handle = asset_server.load("PlayerFishy(96x80).png");
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::new(consts::PLAYER_SPRITE_WIDTH, consts::PLAYER_SPRITE_HEIGHT),
        14,
        7,
    );
    let atlas_handle = texture_atlases.add(texture_atlas);

    let sharky_texture_handle = asset_server.load("PlayerSharky(96x80).png");
    let sharky_texture_atlas =
        TextureAtlas::from_grid(sharky_texture_handle, Vec2::new(96., 80.), 14, 7);
    let sharky_atlas_handle = texture_atlases.add(sharky_texture_atlas);

    let bandit_texture_handle = asset_server.load("FishFight_BanditAnimation_skin2.png");
    let bandit_texture_atlas =
        TextureAtlas::from_grid(bandit_texture_handle, Vec2::new(64., 64.), 8, 6);
    let bandit_atlas_handle = texture_atlases.add(bandit_texture_atlas);

    let slinger_texture_handle =
        asset_server.load("FishFight_SlingerAnimation_Idle_Walk_Shot_Run_Falling.png");
    let slinger_texture_atlas =
        TextureAtlas::from_grid(slinger_texture_handle, Vec2::new(80., 80.), 8, 6);
    let slinger_atlas_handle = texture_atlases.add(slinger_texture_atlas);

    //Layers mapping to state
    let mut player_animation_map = HashMap::default();
    player_animation_map.insert(State::Idle, 0..13);
    player_animation_map.insert(State::Running, 14..19);
    player_animation_map.insert(State::KnockedRight, 85..90);
    player_animation_map.insert(State::KnockedLeft, 71..76);
    player_animation_map.insert(State::Dying, 71..76);
    player_animation_map.insert(State::Attacking, 85..90);

    let mut bandit_animation_map = HashMap::default();
    bandit_animation_map.insert(State::Idle, 0..7);
    bandit_animation_map.insert(State::Running, 8..15);
    bandit_animation_map.insert(State::KnockedRight, 40..46);
    bandit_animation_map.insert(State::KnockedLeft, 40..46);
    bandit_animation_map.insert(State::Dying, 40..46);
    bandit_animation_map.insert(State::Attacking, 16..23);

    let mut slinger_animation_map = HashMap::default();
    slinger_animation_map.insert(State::Idle, 0..3);
    slinger_animation_map.insert(State::Running, 8..11);
    slinger_animation_map.insert(State::KnockedRight, 40..46);
    slinger_animation_map.insert(State::KnockedLeft, 40..46);
    slinger_animation_map.insert(State::Dying, 40..46);
    slinger_animation_map.insert(State::Attacking, 16..20);

    //Insert player
    commands
        .spawn_bundle(PlayerBundle { ..default() })
        .insert_bundle(AnimatedSpriteSheetBundle {
            sprite_sheet: SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(0),
                texture_atlas: atlas_handle,
                transform: Transform::from_xyz(0., consts::GROUND_Y, 0.),
                ..Default::default()
            },
            animation: Animation::new(7. / 60., player_animation_map.clone()),
        })
        .insert_bundle(CharacterBundle { ..default() })
        .insert_bundle(PhysicsBundle {
            collision_groups: CollisionGroups::new(BodyLayers::Player as u32, 0b1111),
            ..default()
        });

    //Insert sharky "enemy"
    commands
        .spawn_bundle(EnemyBundle { ..default() })
        .insert_bundle(AnimatedSpriteSheetBundle {
            sprite_sheet: SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(0),
                texture_atlas: sharky_atlas_handle.clone(),
                transform: Transform::from_xyz(100., consts::GROUND_Y + 20., 0.),
                ..Default::default()
            },
            animation: Animation::new(7. / 60., player_animation_map.clone()),
        })
        .insert_bundle(CharacterBundle { ..default() })
        .insert_bundle(PhysicsBundle {
            collision_groups: CollisionGroups::new(BodyLayers::Enemy as u32, 0b1111),
            ..default()
        });

    //Insert slinger enemy
    commands
        .spawn_bundle(EnemyBundle { ..default() })
        .insert_bundle(AnimatedSpriteSheetBundle {
            sprite_sheet: SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(0),
                texture_atlas: slinger_atlas_handle.clone(),
                transform: Transform::from_xyz(250., consts::GROUND_Y + 45., 0.),
                ..Default::default()
            },
            animation: Animation::new(7. / 60., slinger_animation_map.clone()),
        })
        .insert_bundle(CharacterBundle { ..default() })
        .insert_bundle(PhysicsBundle {
            collision_groups: CollisionGroups::new(BodyLayers::Enemy as u32, 0b1111),
            ..default()
        });

    commands
        .spawn_bundle(EnemyBundle { ..default() })
        .insert_bundle(AnimatedSpriteSheetBundle {
            sprite_sheet: SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(0),
                texture_atlas: bandit_atlas_handle.clone(),
                transform: Transform::from_xyz(400., consts::GROUND_Y - 15., 0.),
                ..Default::default()
            },
            animation: Animation::new(7. / 60., bandit_animation_map.clone()),
        })
        .insert_bundle(CharacterBundle { ..default() })
        .insert_bundle(PhysicsBundle {
            collision_groups: CollisionGroups::new(BodyLayers::Enemy as u32, 0b1111),
            ..default()
        });
    /*    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("floor.png"),
        transform: Transform::from_xyz(0., consts::GROUND_Y, 5.),
        ..Default::default()
    }); */
}

fn player_attack(
    mut query: Query<(&mut State, &mut Transform, &Animation, &Facing), With<Player>>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (mut state, mut transform, animation, facing) = query.single_mut();

    if *state != State::Attacking {
        if keyboard.just_pressed(KeyCode::Space) {
            state.set(State::Attacking);
        }
    } else {
        if animation.is_finished() {
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
