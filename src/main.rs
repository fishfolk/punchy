use std::ops::Range;

use bevy::prelude::*;

#[derive(Component)]
struct Player {
    movement_speed: f32,
    facing_left: bool,
}

#[derive(Component)]
struct Animation {
    indices: Range<usize>,
    timer: Timer,
}

impl Animation {
    fn new(fps: f32, indices: Range<usize>) -> Self {
        Self {
            indices: indices,
            timer: Timer::from_seconds(fps, true),
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(player_controller)
        .add_system(animation_cycling)
        .add_system(animation_flipping_system)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    let texture_handle = asset_server.load("PlayerFishy(96x80).png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(96., 80.), 14, 7);
    let atlas_handle = texture_atlases.add(texture_atlas);

    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: atlas_handle,
            //transform: Transform::from_xyz(0., 0., 1.),
            ..Default::default()
        })
        .insert(Player {
            movement_speed: 250.0,
            facing_left: false,
        })
        .insert(Animation::new(5. / 60., 0..13));
}

fn player_controller(
    mut query: Query<(&mut Player, &mut Transform)>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (mut player, mut transform) = query.single_mut();

    if keyboard.pressed(KeyCode::A) {
        transform.translation.x -= player.movement_speed * time.delta_seconds();
        player.facing_left = true;
    }

    if keyboard.pressed(KeyCode::D) {
        transform.translation.x += player.movement_speed * time.delta_seconds();
        player.facing_left = false;
    }

    if keyboard.pressed(KeyCode::W) {
        transform.translation.y += player.movement_speed * time.delta_seconds();
    }

    if keyboard.pressed(KeyCode::S) {
        transform.translation.y -= player.movement_speed * time.delta_seconds();
    }
}

fn animation_cycling(mut query: Query<(&mut TextureAtlasSprite, &mut Animation)>, time: Res<Time>) {
    for (mut texture_atlas_sprite, mut animation) in query.iter_mut() {
        animation.timer.tick(time.delta());

        if animation.timer.finished() {
            animation.timer.reset();
            texture_atlas_sprite.index += 1;
            if texture_atlas_sprite.index > animation.indices.end {
                texture_atlas_sprite.index = animation.indices.start;
            }
        }
    }
}

fn animation_flipping_system(mut query: Query<(&mut TextureAtlasSprite, &Player)>) {
    let (mut texture_atlas_sprite, player) = query.single_mut();

    texture_atlas_sprite.flip_x = player.facing_left;
}
