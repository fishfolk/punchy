use std::ops::Range;

use heron::{prelude::*, SensorShape};

use bevy::{prelude::*, render::camera::CameraProjection, utils::HashMap};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum State {
    IDLE,
    RUNNING,
    ATTACKING,
    KNOCKED,
    DYING,
}

#[derive(PhysicsLayer)]
enum BodyLayers {
    Enemy,
    Player,
}

#[derive(Component)]
struct Player {
    movement_speed: f32,
    facing_left: bool,
    state: State,
}

#[derive(Component)]
struct Animation {
    animations: HashMap<State, Range<usize>>,
    current_frame: usize,
    current_state: Option<State>,
    timer: Timer,
    played_once: bool,
}

impl Animation {
    fn new(fps: f32, animations: HashMap<State, Range<usize>>) -> Self {
        Self {
            animations,
            current_frame: 0,
            current_state: None,
            timer: Timer::from_seconds(fps, false),
            played_once: false,
        }
    }

    fn set(&mut self, state: State) {
        if self.current_state == Some(state) {
            return;
        }

        self.played_once = false;
        self.current_frame = 0;
        self.current_state = Some(state);
        self.timer.reset();
    }

    fn is_finished(&self) -> bool {
        self.played_once
    }

    fn is_last_frame(&self) -> bool {
        if let Some(indices) = self.get_current_indices() {
            if let Some(index) = self.get_current_index() {
                return index >= indices.end;
            }
        }

        return false;
    }

    fn get_current_indices(&self) -> Option<&Range<usize>> {
        if let Some(state) = self.current_state {
            return self.animations.get(&state);
        }

        None
    }

    fn get_current_index(&self) -> Option<usize> {
        if let Some(indices) = self.get_current_indices() {
            return Some(indices.start + self.current_frame);
        }

        None
    }
}

#[derive(Component)]
struct YSort(f32);

#[derive(Component)]
struct Stats {
    health: i32,
    damage: i32,
}

#[derive(Component)]
pub struct Parallax;

const PLAYER_WIDTH: f32 = 96.;
const PLAYER_HEIGHT: f32 = 80.;
const PLAYER_HITBOX_HEIGHT: f32 = 50.;

const GROUND_HEIGHT: f32 = 130.;
const GROUND_OFFSET: f32 = 25.;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.494, 0.658, 0.650)))
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        .add_startup_system(setup)
        .add_system(player_controller)
        .add_system(camera_follow_player)
        .add_system(animation_cycling)
        .add_system(animation_flipping)
        .add_system(player_animation_state)
        .add_system(parallax_system)
        .add_system(player_attack)
        .add_system(helper_camera_controller)
        .add_system(y_sort)
        .add_system(knock_enemies)
        .add_system(kill_entities)
        .add_system(knocked_state)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    // camera_bundle.orthographic_projection.depth_calculation = DepthCalculation::Distance;
    camera_bundle.orthographic_projection.scale = 0.75;
    commands.spawn_bundle(camera_bundle);

    let texture_handle = asset_server.load("PlayerFishy(96x80).png");
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT),
        14,
        7,
    );
    let atlas_handle = texture_atlases.add(texture_atlas);

    let mut animation_map = HashMap::default();

    animation_map.insert(State::IDLE, 0..13);
    animation_map.insert(State::RUNNING, 14..19);
    animation_map.insert(State::KNOCKED, 71..76);
    animation_map.insert(State::DYING, 71..76);
    animation_map.insert(State::ATTACKING, 85..90);

    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: atlas_handle,
            transform: Transform::from_xyz(0., 0., 0.),
            ..Default::default()
        })
        .insert(Player {
            movement_speed: 150.0,
            facing_left: false,
            state: State::IDLE,
        })
        .insert(Stats {
            health: 100,
            damage: 35,
        })
        .insert(RigidBody::Sensor)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(PLAYER_WIDTH, PLAYER_HITBOX_HEIGHT, 0.) / 8.,
            border_radius: None,
        })
        .insert(CollisionLayers::new(BodyLayers::Player, BodyLayers::Enemy))
        .insert(SensorShape)
        .insert(Animation::new(7. / 60., animation_map.clone()))
        .insert(YSort(100.));

    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("preview_stage.png"),
        transform: Transform::from_xyz(0., 0., 0.),
        ..Default::default()
    });

    let enemy_texture_handle = asset_server.load("PlayerSharky(96x80).png");
    let enemy_texture_atlas =
        TextureAtlas::from_grid(enemy_texture_handle, Vec2::new(96., 80.), 14, 7);
    let enemy_atlas_handle = texture_atlases.add(enemy_texture_atlas);

    for pos in vec![(100., 35.), (300., -65.)] {
        commands
            .spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(0),
                texture_atlas: enemy_atlas_handle.clone(),
                transform: Transform::from_xyz(pos.0, pos.1, 0.),
                ..Default::default()
            })
            .insert(Stats {
                health: 100,
                damage: 35,
            })
            .insert(RigidBody::Sensor)
            .insert(CollisionShape::Cuboid {
                half_extends: Vec3::new(PLAYER_WIDTH, PLAYER_HITBOX_HEIGHT, 0.) / 8.,
                border_radius: None,
            })
            .insert(CollisionLayers::new(BodyLayers::Enemy, BodyLayers::Player))
            .insert(Animation::new(7. / 60., animation_map.clone()))
            .insert(YSort(100.));
    }

    /* commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("background_01.png"),
            transform: Transform::from_xyz(0., 0., 4.),
            ..Default::default()
        })
        .insert(Parallax);

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("background_02.png"),
            transform: Transform::from_xyz(0., 0., 3.),
            ..Default::default()
        })
        .insert(Parallax);

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("background_03.png"),
            transform: Transform::from_xyz(0., 0., 2.),
            ..Default::default()
        })
        .insert(Parallax); */
}

fn player_controller(
    mut query: Query<(&mut Player, &mut Transform)>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (mut player, mut transform) = query.single_mut();

    if player.state == State::ATTACKING {
        return;
    }

    let mut dir = Vec2::ZERO;

    if keyboard.pressed(KeyCode::A) {
        dir -= Vec2::X;
    }

    if keyboard.pressed(KeyCode::D) {
        dir += Vec2::X;
    }

    if keyboard.pressed(KeyCode::W) {
        dir += Vec2::Y;
    }

    if keyboard.pressed(KeyCode::S) {
        dir -= Vec2::Y;
    }

    //Normalize direction
    dir = dir.normalize_or_zero() * player.movement_speed * time.delta_seconds();

    //Restrict player to the ground
    let new_y = transform.translation.y + dir.y;
    let max_y = GROUND_HEIGHT / 2.;

    if new_y + GROUND_OFFSET >= max_y || new_y + GROUND_OFFSET <= -max_y {
        dir.y = 0.;
    }

    //Move the player
    transform.translation.x += dir.x;
    transform.translation.y += dir.y;

    //Set the player state and direction
    if dir.x > 0. {
        player.facing_left = false;
    } else if dir.x < 0. {
        player.facing_left = true;
    }

    if dir == Vec2::ZERO {
        player.state = State::IDLE;
    } else {
        player.state = State::RUNNING;
    }
}

fn animation_cycling(mut query: Query<(&mut TextureAtlasSprite, &mut Animation)>, time: Res<Time>) {
    //TODO: Add a tick method on Animation
    for (mut texture_atlas_sprite, mut animation) in query.iter_mut() {
        animation.timer.tick(time.delta());

        if animation.timer.finished() {
            animation.timer.reset();

            if animation.is_last_frame() {
                animation.played_once = true; // Check if animation player here because we need to wait the last frame
                animation.current_frame = 0;
            } else {
                animation.current_frame += 1;
            }
        }

        if let Some(index) = animation.get_current_index() {
            texture_atlas_sprite.index = index;
        }
    }
}

fn animation_flipping(mut query: Query<(&mut TextureAtlasSprite, &Player)>) {
    let (mut texture_atlas_sprite, player) = query.single_mut();

    texture_atlas_sprite.flip_x = player.facing_left;
}

fn player_animation_state(mut query: Query<(&Player, &mut Animation)>) {
    let (player, mut animation) = query.single_mut();

    animation.set(player.state);
}

fn camera_follow_player(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    time: Res<Time>,
) {
    let player = player_query.single().translation;
    let mut camera = camera_query.single_mut();

    //TODO: Add a way to change the camera speed
    camera.translation.x += (player.x - camera.translation.x) * time.delta_seconds() * 5.;
    //   camera.translation.y += (player.y - camera.translation.y) * time.delta_seconds() * 5.;
}

fn parallax_system(
    cam_query: Query<&Transform, With<Camera>>,
    mut query: Query<&mut Transform, (With<Parallax>, Without<Camera>)>,
) {
    let cam_trans = cam_query.single();

    //TODO: Check the parallax values
    for mut trans in query.iter_mut() {
        trans.translation.x = -cam_trans.translation.x * (0.2 * trans.translation.z);
        trans.translation.y = -cam_trans.translation.y * (0.1 * trans.translation.z);
    }
}

fn player_attack(
    mut query: Query<(&mut Player, &mut Transform, &Animation)>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (mut player, mut transform, animation) = query.single_mut();

    if player.state != State::ATTACKING {
        if keyboard.just_pressed(KeyCode::Space) {
            player.state = State::ATTACKING;
        }
    } else {
        if animation.is_finished() {
            player.state = State::IDLE;
        } else {
            //TODO: Fix hacky way to get a forward jump
            if animation.current_frame < 3 {
                if player.facing_left {
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

//Helper camera controller
pub fn helper_camera_controller(
    mut query: Query<(&mut Camera, &mut OrthographicProjection, &mut Transform)>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    windows: Res<Windows>,
) {
    let (mut camera, mut projection, mut transform) = query.single_mut();

    if keys.pressed(KeyCode::Up) {
        transform.translation.y += 150.0 * time.delta_seconds();
    }
    if keys.pressed(KeyCode::Left) {
        transform.translation.x -= 150.0 * time.delta_seconds();
    }
    if keys.pressed(KeyCode::Down) {
        transform.translation.y -= 150.0 * time.delta_seconds();
    }
    if keys.pressed(KeyCode::Right) {
        transform.translation.x += 150.0 * time.delta_seconds();
    }

    //println!("{:?}", transform.translation);

    let scale = projection.scale;

    let w = windows.get(camera.window).unwrap();

    if keys.pressed(KeyCode::Z) {
        projection.scale -= 0.55 * time.delta_seconds();
    }
    if keys.pressed(KeyCode::X) {
        projection.scale += 0.55 * time.delta_seconds();
    }

    if (projection.scale - scale).abs() > f32::EPSILON {
        projection.update(w.width(), w.height());
        camera.projection_matrix = projection.get_projection_matrix();
        camera.depth_calculation = projection.depth_calculation();
    }
}

fn y_sort(mut query: Query<(&mut Transform, &YSort)>) {
    for (mut transform, ysort) in query.iter_mut() {
        transform.translation.z = ysort.0 - transform.translation.y;
    }
}

fn knock_enemies(
    mut events: EventReader<CollisionEvent>,
    mut query: Query<(&mut Animation, &mut Stats)>,
) {
    events.iter().filter(|e| e.is_started()).for_each(|e| {
        let (e1, e2) = e.rigid_body_entities();
        let (l1, l2) = e.collision_layers();

        if l1.contains_group(BodyLayers::Player) && l2.contains_group(BodyLayers::Enemy) {
            let (player_anim, player_stats) = query.get(e1).unwrap();
            if let Ok((mut anim, mut stats)) = query.get_mut(e2) {
                if player_anim.current_state == Some(State::ATTACKING) {
                    stats.health = stats.health - player_stats.damage;
                    anim.set(State::KNOCKED);
                }
            }
        }
    })
}

fn kill_entities(mut commands: Commands, mut query: Query<(Entity, &Stats, &mut Animation)>) {
    for (entity, stats, mut animation) in query.iter_mut() {
        if stats.health <= 0 {
            animation.set(State::DYING);
        }

        if animation.current_state == Some(State::DYING) && animation.is_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn knocked_state(mut query: Query<&mut Animation>) {
    for mut animation in query.iter_mut() {
        if animation.current_state == Some(State::KNOCKED) && animation.is_finished() {
            animation.set(State::IDLE);
        }
    }
}
