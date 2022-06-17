use bevy::{prelude::*, render::camera::ScalingMode, utils::HashMap};
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

#[derive(Component)]
pub struct DespawnMarker;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.494, 0.658, 0.650)))
        .insert_resource(WindowDescriptor {
            title: "Fish Fight Punchy".to_string(),
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
        // .add_system(camera_follow_player)
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
        .add_system_to_stage(CoreStage::Last, despawn_entities)
        .run();
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

    //Layers mapping to state
    let mut animation_map = HashMap::default();
    animation_map.insert(State::Idle, 0..13);
    animation_map.insert(State::Running, 14..19);
    animation_map.insert(State::KnockedRight, 85..90);
    animation_map.insert(State::KnockedLeft, 71..76);
    animation_map.insert(State::Dying, 71..76);
    animation_map.insert(State::Attacking, 85..90);

    //Insert player
    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: atlas_handle,
            transform: Transform::from_xyz(0., consts::GROUND_Y, 0.),
            ..Default::default()
        })
        .insert(Player)
        .insert(State::Idle)
        .insert(Stats {
            //TODO: Store default stats in consts
            health: 100,
            damage: 35,
            movement_speed: 150.0,
        })
        .insert(Facing::Right)
        .insert(Collider::cuboid(
            consts::PLAYER_SPRITE_WIDTH / 8.,
            consts::PLAYER_HITBOX_HEIGHT / 8.,
        ))
        .insert(Sensor(true))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC)
        .insert(CollisionGroups::new(BodyLayers::Player as u32, 0b1111))
        .insert(Animation::new(7. / 60., animation_map.clone()))
        .insert(YSort(100.));

    let enemy_texture_handle = asset_server.load("PlayerSharky(96x80).png");
    let enemy_texture_atlas =
        TextureAtlas::from_grid(enemy_texture_handle, Vec2::new(96., 80.), 14, 7);
    let enemy_atlas_handle = texture_atlases.add(enemy_texture_atlas);

    //Insert enemies
    for pos in vec![
        (100., consts::GROUND_Y + 25.),
        (400., consts::GROUND_Y - 15.),
    ] {
        commands
            .spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(0),
                texture_atlas: enemy_atlas_handle.clone(),
                transform: Transform::from_xyz(pos.0, pos.1, 0.),
                ..Default::default()
            })
            .insert(Enemy)
            .insert(State::Idle)
            .insert(Facing::Left)
            .insert(Stats {
                health: 100,
                damage: 35,
                movement_speed: 120.0,
            })
            .insert(Collider::cuboid(
                consts::PLAYER_SPRITE_WIDTH / 8.,
                consts::PLAYER_HITBOX_HEIGHT / 8.,
            ))
            .insert(Sensor(true))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC)
            .insert(CollisionGroups::new(BodyLayers::Enemy as u32, 0b1111))
            .insert(Animation::new(7. / 60., animation_map.clone()))
            .insert(YSort(100.));
    }

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
