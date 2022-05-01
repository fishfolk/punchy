use bevy::{prelude::*, render::camera::ScalingMode, utils::HashMap};
use bevy_parallax::{LayerData, ParallaxCameraComponent, ParallaxPlugin, ParallaxResource};
use heron::{prelude::*, SensorShape};

mod animation;
mod attack;
mod camera;
mod collisions;
mod consts;
mod movement;
mod state;
mod y_sort;

use animation::*;
use attack::AttackPlugin;
use camera::*;
use collisions::*;
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

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.494, 0.658, 0.650)))
        .insert_resource(WindowDescriptor {
            title: "Fish Fight Punchy".to_string(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(AttackPlugin)
        .add_plugin(AnimationPlugin)
        .add_plugin(StatePlugin)
        .insert_resource(ParallaxResource {
            layer_data: vec![
                LayerData {
                    speed: 0.98,
                    path: "background_03.png".to_string(),
                    tile_size: Vec2::new(896.0, 480.0),
                    cols: 1,
                    rows: 1,
                    z: 0.0,
                    scale: 1.2,
                    ..Default::default()
                },
                LayerData {
                    speed: 0.9,
                    path: "background_02.png".to_string(),
                    tile_size: Vec2::new(896.0, 480.0),
                    cols: 1,
                    rows: 1,
                    z: 1.0,
                    scale: 1.2,
                    ..Default::default()
                },
                LayerData {
                    speed: 0.82,
                    path: "background_01.png".to_string(),
                    tile_size: Vec2::new(896.0, 480.0),
                    cols: 1,
                    rows: 1,
                    z: 2.0,
                    scale: 1.2,
                    ..Default::default()
                },
            ],
            ..Default::default()
        })
        .add_plugin(ParallaxPlugin)
        .add_startup_system(setup)
        .add_system(player_controller)
        .add_system(camera_follow_player)
        .add_system(player_attack)
        .add_system(helper_camera_controller)
        .add_system(y_sort)
        // .add_system(knock_enemies)
        .add_system(player_attack_collision)
        .add_system(kill_entities)
        .add_system(knockback_system)
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
    camera_bundle.orthographic_projection.scale = 16. * 14.5;
    commands
        .spawn_bundle(camera_bundle)
        .insert(ParallaxCameraComponent);

    let texture_handle = asset_server.load("PlayerFishy(96x80).png");
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::new(consts::PLAYER_WIDTH, consts::PLAYER_HEIGHT),
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
        .insert(RigidBody::Sensor)
        .insert(Collisions::default())
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(consts::PLAYER_WIDTH, consts::PLAYER_HITBOX_HEIGHT, 0.) / 8.,
            border_radius: None,
        })
        .insert(CollisionLayers::new(BodyLayers::Player, BodyLayers::Enemy))
        .insert(SensorShape)
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
            .insert(RigidBody::Sensor)
            .insert(Collisions::default())
            .insert(CollisionShape::Cuboid {
                half_extends: Vec3::new(consts::PLAYER_WIDTH, consts::PLAYER_HITBOX_HEIGHT, 0.)
                    / 8.,
                border_radius: None,
            })
            .insert(CollisionLayers::new(BodyLayers::Enemy, BodyLayers::Player))
            .insert(Animation::new(7. / 60., animation_map.clone()))
            .insert(YSort(100.));
    }

    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("floor.png"),
        transform: Transform::from_xyz(0., consts::GROUND_Y, 5.),
        ..Default::default()
    });
}

fn player_attack(
    mut query: Query<(&mut State, &mut Transform, &Animation, &Facing), With<Player>>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (mut state, mut transform, animation, facing) = query.single_mut();

    if *state != State::Attacking {
        if keyboard.just_pressed(KeyCode::Space) {
            *state = State::Attacking;
        }
    } else {
        if animation.is_finished() {
            *state = State::Idle;
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

/* fn damage_on_collision( mut events: EventReader<CollisionEvent>,
    mut set: ParamSet<(Query<&mut Stats>, Query<&Damage>)>,
    mut commands: Commands){
        events.iter().filter(|e| e.is_started()).for_each(|e| {
            let ( stats_entity,  damage_entity) = e.rigid_body_entities();

         /*    if set.p0().contains(damage_entity) && set.p1().contains(stats_entity){
                let temp = stats_entity;
                stats_entity = damage_entity;
                damage_entity = temp;
            }  */
            let p0 = set.p0();
            let p1 = set.p1();
           let stats = p0.get_mut(stats_entity);
           let damage = p1.get(damage_entity);

            if let Ok(stats) = stats {
                if let Ok(damage) = damage{
                    stats.health -= damage.0;
                    if stats.health <= 0 {
                        commands.entity(stats_entity).despawn_recursive();
                    }
                }
            }

        });
} */

fn kill_entities(
    mut commands: Commands,
    mut query: Query<(Entity, &Stats, &Animation, &mut State)>,
) {
    for (entity, stats, animation, mut state) in query.iter_mut() {
        if stats.health <= 0 {
            *state = State::Dying;
        }

        if *state == State::Dying && animation.is_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
