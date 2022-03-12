use std::ops::Range;

use bevy::{prelude::*, utils::HashMap};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum State {
    IDLE,
    RUNNING,
    ATTACKING,
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
}

impl Animation {
    fn new(fps: f32, animations: HashMap<State, Range<usize>>) -> Self {
        Self {
            animations,
            current_frame: 0,
            current_state: None,
            timer: Timer::from_seconds(fps, true),
        }
    }

    fn set(&mut self, state: State) {
        if self.current_state == Some(state) {
            return;
        }

        self.current_frame = 0;
        self.current_state = Some(state);
        self.timer.reset();
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
pub struct Parallax;
fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.494, 0.658, 0.650)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(player_controller)
        .add_system(camera_follow_player)
        .add_system(animation_cycling)
        .add_system(animation_flipping)
        .add_system(player_animation_state)
        .add_system(parallax_system)
        .add_system(player_attack)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.scale = 0.75;
    commands.spawn_bundle(camera_bundle);

    let texture_handle = asset_server.load("PlayerFishy(96x80).png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(96., 80.), 14, 7);
    let atlas_handle = texture_atlases.add(texture_atlas);

    let mut animation_map = HashMap::default();

    animation_map.insert(State::IDLE, 0..13);
    animation_map.insert(State::RUNNING, 14..19);
    animation_map.insert(State::ATTACKING, 85..91);

    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: atlas_handle,
            transform: Transform::from_xyz(0., 0., 100.),
            ..Default::default()
        })
        .insert(Player {
            movement_speed: 150.0,
            facing_left: false,
            state: State::IDLE,
        })
        .insert(Animation::new(7. / 60., animation_map));

    commands
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
        .insert(Parallax);
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

    dir = dir.normalize_or_zero() * player.movement_speed * time.delta_seconds();
    transform.translation.x += dir.x;
    transform.translation.y += dir.y;

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
    for (mut texture_atlas_sprite, mut animation) in query.iter_mut() {
        animation.timer.tick(time.delta());

        if animation.timer.finished() {
            animation.timer.reset();

            if animation.is_last_frame() {
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
    camera.translation.y += (player.y - camera.translation.y) * time.delta_seconds() * 5.;
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
        if animation.is_last_frame() {
            player.state = State::IDLE;
        } else {
            //TODO: Fix hacky way to get a forward jump
            if animation.current_frame < 3 {
                if player.facing_left {
                    transform.translation.x -= 100. * time.delta_seconds();
                } else {
                    transform.translation.x += 100. * time.delta_seconds();
                }
            } else if animation.current_frame < 4 {
                if player.facing_left {
                    transform.translation.x -= 50. * time.delta_seconds();
                } else {
                    transform.translation.x += 50. * time.delta_seconds();
                }
            }

            if animation.current_frame < 1 {
                transform.translation.y += 140. * time.delta_seconds();
            } else if animation.current_frame < 3 {
                transform.translation.y -= 70. * time.delta_seconds();
            }
        }
    }
}
