use std::time::Duration;

use bevy::{
    core::{Time, Timer},
    hierarchy::{BuildChildren, Children, DespawnRecursiveExt},
    math::{Vec2, Vec3},
    prelude::{
        default, App, AssetServer, Assets, Bundle, Commands, Component, Entity, EventReader,
        Handle, Local, Parent, Plugin, Query, Res, Transform, With, Without,
    },
    sprite::SpriteBundle,
    transform::TransformBundle,
};
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    animation::{Animation, Facing},
    audio::FighterStateEffectsPlayback,
    collisions::BodyLayers,
    consts::{
        self, ATTACK_HEIGHT, ATTACK_LAYER, ATTACK_WIDTH, ITEM_BOTTLE_NAME, ITEM_HEIGHT, ITEM_LAYER,
        ITEM_WIDTH, THROW_ITEM_ROTATION_SPEED,
    },
    enemy::Boss,
    input::PlayerAction,
    item::item_carried_by_player,
    metadata::{FighterMeta, GameMeta, ItemMeta},
    movement::{
        clamp_player_movements, LeftMovementBoundary, MoveInArc, MoveInDirection, Rotate, Target,
    },
    state::State,
    ArrivedEvent, Enemy, GameState, Player, Stats,
};

pub struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(player_projectile_attack)
                .with_system(player_throw)
                .with_system(player_flop)
                .with_system(boss_jump_attack)
                .with_system(activate_hitbox)
                .with_system(deactivate_hitbox)
                .with_system(projectile_cleanup)
                .with_system(projectile_tick)
                .into(),
        )
        .add_system(
            enemy_attack
                .run_in_state(GameState::InGame)
                .after("move_to_target"),
        )
        .add_system(
            boss_attack
                .run_in_state(GameState::InGame)
                .after("move_to_target"),
        );
    }
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component)]
pub struct Attack {
    pub damage: i32,
}

#[derive(Component)]
pub struct AttackFrames {
    pub startup: usize,
    pub active: usize,
    pub recovery: usize,
}

#[derive(Component)]
pub struct ProjectileLifetime(pub Timer);

#[derive(Bundle)]
pub struct Projectile {
    #[bundle]
    sprite_bundle: SpriteBundle,
    rotate: Rotate,
    collider: Collider,
    sensor: Sensor,
    events: ActiveEvents,
    collision_types: ActiveCollisionTypes,
    collision_groups: CollisionGroups,
    facing: Facing,
    move_in_direction: MoveInDirection,
    attack: Attack,
    attack_timer: ProjectileLifetime,
}

impl Projectile {
    pub fn new(
        transform: &Transform,
        facing: &Facing,
        dir: Vec2,
        asset_server: &Res<AssetServer>,
    ) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                texture: asset_server.load("bottled_seaweed11x31.png"),
                transform: Transform::from_xyz(
                    transform.translation.x,
                    transform.translation.y,
                    ATTACK_LAYER,
                ),
                ..default()
            },
            rotate: Rotate {
                speed: THROW_ITEM_ROTATION_SPEED,
                to_right: !facing.is_left(),
            },
            collider: Collider::cuboid(ATTACK_WIDTH / 2., ATTACK_HEIGHT / 2.),
            sensor: Sensor(true),
            events: ActiveEvents::COLLISION_EVENTS,
            collision_types: ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC,
            //TODO: define collision layer based on the fighter shooting projectile, load for asset files of fighter which "team" they are on
            collision_groups: CollisionGroups::new(BodyLayers::PLAYER_ATTACK, BodyLayers::ENEMY),
            facing: facing.clone(),
            move_in_direction: MoveInDirection(dir * 300.), //TODO: Put the velocity in a cons,
            attack: Attack { damage: 10 },
            attack_timer: ProjectileLifetime(Timer::new(Duration::from_secs(1), false)),
        }
    }
}

#[derive(Bundle)]
pub struct ThrownItem {
    #[bundle]
    sprite_bundle: SpriteBundle,
    rotate: Rotate,
    move_in_arc: MoveInArc,
    collider: Collider,
    sensor: Sensor,
    events: ActiveEvents,
    collision_types: ActiveCollisionTypes,
    collision_groups: CollisionGroups,
    attack: Attack,
}

impl ThrownItem {
    pub fn new(
        angles: (f32, f32),
        position: Vec2,
        facing: Facing,
        asset_server: &AssetServer,
    ) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                texture: asset_server.load("bottled_seaweed11x31.png"),
                transform: Transform::from_xyz(position.x, position.y, ITEM_LAYER),
                ..default()
            },
            rotate: Rotate {
                speed: consts::THROW_ITEM_ROTATION_SPEED,
                to_right: !facing.is_left(),
            },
            move_in_arc: MoveInArc {
                //TODO: Set in consts
                radius: Vec2::new(
                    50.,
                    consts::PLAYER_HEIGHT + consts::THROW_ITEM_Y_OFFSET + consts::ITEM_HEIGHT,
                ),
                speed: consts::THROW_ITEM_SPEED,
                angle: angles.0,
                end_angle: angles.1,
                inverse_direction: facing.is_left(),
                origin: position,
            },
            collider: Collider::cuboid(ITEM_WIDTH / 2., ITEM_HEIGHT / 2.),
            sensor: Sensor(true),
            events: ActiveEvents::COLLISION_EVENTS,
            collision_types: ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC,
            //TODO: define collision layer based on the fighter throwing the item, load for asset files of fighter which "team" they are on
            collision_groups: CollisionGroups::new(BodyLayers::ITEM, BodyLayers::ENEMY),
            attack: Attack {
                damage: consts::THROW_ITEM_DAMAGE,
            },
        }
    }
}

fn player_projectile_attack(
    player_query: Query<
        (
            &Children,
            &Transform,
            &Facing,
            &State,
            &ActionState<PlayerAction>,
        ),
        With<Player>,
    >,
    items_meta_query: Query<&Handle<ItemMeta>>,
    items_meta: Res<Assets<ItemMeta>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (player_children, transform, facing, state, input) in player_query.iter() {
        if *state != State::Idle && *state != State::Running {
            continue;
        }

        let carried_item = item_carried_by_player(
            player_children,
            ITEM_BOTTLE_NAME,
            &items_meta_query,
            &items_meta,
        );

        if let Some(bottle_id) = carried_item {
            if input.just_pressed(PlayerAction::Shoot) {
                let mut dir = Vec2::X;

                if facing.is_left() {
                    dir = -dir;
                }

                let projectile = Projectile::new(transform, facing, dir, &asset_server);

                commands.spawn_bundle(projectile);

                commands.entity(bottle_id).despawn();
            }
        }
    }
}

fn player_throw(
    mut commands: Commands,
    player_query: Query<
        (
            &Children,
            &Transform,
            Option<&Facing>,
            &ActionState<PlayerAction>,
        ),
        With<Player>,
    >,
    items_meta_query: Query<&Handle<ItemMeta>>,
    items_meta: Res<Assets<ItemMeta>>,
    asset_server: Res<AssetServer>,
) {
    for (player_children, transform, facing_option, input) in player_query.iter() {
        let carried_item = item_carried_by_player(
            player_children,
            ITEM_BOTTLE_NAME,
            &items_meta_query,
            &items_meta,
        );

        if let Some(bottle_id) = carried_item {
            if input.just_pressed(PlayerAction::Throw) {
                let facing = match facing_option {
                    Some(f) => f.clone(),
                    None => Facing::Right,
                };

                let mut position = transform.translation.truncate();

                //Offset the position depending on the facing
                if facing.is_left() {
                    position.x -= consts::THROW_ITEM_X_OFFSET;
                } else {
                    position.x += consts::THROW_ITEM_X_OFFSET;
                }

                position.y -= consts::PLAYER_HEIGHT / 2.; //Set to the player feet

                let angles = match facing {
                    Facing::Left => (90. - consts::THROW_ITEM_ANGLE_OFFSET, 180.),
                    Facing::Right => (90. + consts::THROW_ITEM_ANGLE_OFFSET, 0.),
                };

                let thrown_item = ThrownItem::new(angles, position, facing, &asset_server);

                commands.spawn_bundle(thrown_item);

                commands.entity(bottle_id).despawn()
            }
        }
    }
}

fn player_flop(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut State,
            &mut Transform,
            &Stats,
            &Animation,
            &Facing,
            &ActionState<PlayerAction>,
            &Handle<FighterMeta>,
        ),
        With<Player>,
    >,
    fighter_assets: Res<Assets<FighterMeta>>,
    time: Res<Time>,
    left_movement_boundary: Res<LeftMovementBoundary>,
    game_meta: Res<GameMeta>,
    mut start_y: Local<Option<f32>>,
) {
    let players_movement = query
        .iter_mut()
        .map(
            |(entity, mut state, transform, stats, animation, facing, input, fighter_meta)| {
                if *state != State::Attacking {
                    if !(*state != State::Idle && *state != State::Running)
                        && input.just_pressed(PlayerAction::FlopAttack)
                    {
                        state.set(State::Attacking);

                        let attack_entity = commands
                            .spawn_bundle(TransformBundle::default())
                            .insert(Sensor(true))
                            .insert(ActiveEvents::COLLISION_EVENTS)
                            .insert(
                                ActiveCollisionTypes::default()
                                    | ActiveCollisionTypes::STATIC_STATIC,
                            )
                            .insert(CollisionGroups::new(
                                BodyLayers::PLAYER_ATTACK,
                                BodyLayers::ENEMY,
                            ))
                            .insert(Attack {
                                damage: stats.damage,
                            })
                            .insert(AttackFrames {
                                startup: 0,
                                active: 3,
                                recovery: 4,
                            })
                            .id();
                        commands.entity(entity).push_children(&[attack_entity]);
                        //TODO: define hitbox size and placement through resources

                        //maybe move audio effects?
                        if let Some(fighter) = fighter_assets.get(fighter_meta) {
                            if let Some(effects) = fighter.audio.effect_handles.get(&state) {
                                let fx_playback =
                                    FighterStateEffectsPlayback::new(*state, effects.clone());
                                commands.entity(entity).insert(fx_playback);
                            }
                        }
                        // commands.
                        // commands.entity(entity)
                    }

                    (transform.translation, None)
                } else {
                    let mut movement = Vec2::ZERO;

                    //TODO: Replace with movement intent eventwriter in movement rewrite!
                    //TODO: Fix hacky way to get a forward jump
                    if animation.current_frame < 3 {
                        if facing.is_left() {
                            movement.x -= 200. * time.delta_seconds();
                        } else {
                            movement.x += 200. * time.delta_seconds();
                        }
                    }

                    // For currently unclear reasons, the first Animation frame may run for less Bevy frames
                    // than expected. When this is the case, the player jumps less then it should, netting,
                    // at the end of the animation, a slightly negative Y than the beginning, which causes
                    // problems. This is a workaround.
                    //
                    if start_y.is_none() {
                        *start_y = Some(transform.translation.y);
                    }

                    if animation.current_frame < 1 {
                        movement.y += 180. * time.delta_seconds();
                    } else if animation.current_frame < 3 {
                        movement.y -= 90. * time.delta_seconds();
                    } else if animation.is_finished() {
                        movement.y = start_y.unwrap();
                        *start_y = None;
                    }

                    (transform.translation, Some(movement))
                }
            },
        )
        .collect::<Vec<_>>();

    let players_movement =
        clamp_player_movements(players_movement, &left_movement_boundary, &game_meta);

    for ((_, _, mut transform, _, _, _, _, _), player_dir) in query.iter_mut().zip(players_movement)
    {
        if let Some(player_dir) = player_dir {
            transform.translation += player_dir.extend(0.);
        }
    }
}

fn enemy_attack(
    mut query: Query<
        (Entity, &mut State, &Facing, &Handle<FighterMeta>),
        (With<Enemy>, With<Target>, Without<Boss>),
    >,
    mut event_reader: EventReader<ArrivedEvent>,
    mut commands: Commands,
    fighter_assets: Res<Assets<FighterMeta>>,
) {
    for event in event_reader.iter() {
        if let Ok((entity, mut state, facing, fighter_handle)) = query.get_mut(event.0) {
            if *state != State::Attacking {
                if rand::random() && *state != State::Waiting {
                    state.set(State::Waiting);
                } else {
                    state.set(State::Attacking);

                    //TODO: remove this offset in favor of offsets defined in fighter assets
                    let offset = match facing {
                        Facing::Left => -24.0,
                        Facing::Right => 24.0,
                    };

                    let attack_entity = commands
                        .spawn_bundle(TransformBundle::from_transform(
                            Transform::from_translation(Vec3::new(offset, 0.0, 0.0)),
                        ))
                        .insert(Sensor(true))
                        .insert(ActiveEvents::COLLISION_EVENTS)
                        .insert(
                            ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC,
                        )
                        .insert(CollisionGroups::new(
                            BodyLayers::ENEMY_ATTACK,
                            BodyLayers::PLAYER,
                        ))
                        .insert(Attack { damage: 10 })
                        .insert(AttackFrames {
                            startup: 1,
                            active: 2,
                            recovery: 3,
                        })
                        .id();
                    commands.entity(event.0).push_children(&[attack_entity]);

                    if let Some(fighter) = fighter_assets.get(fighter_handle) {
                        if let Some(effects) = fighter.audio.effect_handles.get(&state) {
                            let fx_playback =
                                FighterStateEffectsPlayback::new(*state, effects.clone());
                            commands.entity(entity).insert(fx_playback);
                        }
                    }
                }
            }
        }
    }
}

fn boss_attack(
    mut query: Query<
        (Entity, &mut State, &Handle<FighterMeta>),
        (With<Enemy>, With<Target>, With<Boss>),
    >,
    mut event_reader: EventReader<ArrivedEvent>,

    mut commands: Commands,
) {
    for event in event_reader.iter() {
        if let Ok((entity, mut state, fighter_handle)) = query.get_mut(event.0) {
            if *state != State::Attacking {
                if rand::random() && *state != State::Waiting {
                    state.set(State::Waiting);
                } else {
                    state.set(State::Attacking);

                    let attack_entity = commands
                        .spawn_bundle(TransformBundle::from_transform(Transform::default()))
                        .insert(Sensor(true))
                        .insert(ActiveEvents::COLLISION_EVENTS)
                        .insert(
                            ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC,
                        )
                        .insert(CollisionGroups::new(
                            BodyLayers::ENEMY_ATTACK,
                            BodyLayers::PLAYER,
                        ))
                        .insert(Attack { damage: 30 })
                        .insert(AttackFrames {
                            //TODO: Check if this is correct
                            startup: 1,
                            active: 2,
                            recovery: 3,
                        })
                        .id();

                    commands.entity(entity).push_children(&[attack_entity]);
                }
            }
        }
    }
}

fn boss_jump_attack(
    mut query: Query<(&mut State, &Facing, &Animation, &mut Transform)>,
    time: Res<Time>,
    mut start_y: Local<Option<f32>>,
) {
    for (mut state, facing, animation, mut transform) in query.iter_mut() {
        if *state == State::Attacking {
            let mut movement = Vec2::ZERO;

            //TODO: Replace with movement intent eventwriter in movement rewrite!
            //TODO: Fix hacky way to get a forward jump
            if animation.current_frame < 3 {
                if facing.is_left() {
                    movement.x -= 150. * time.delta_seconds();
                } else {
                    movement.x += 150. * time.delta_seconds();
                }
            }

            // For currently unclear reasons, the first Animation frame may run for less Bevy frames
            // than expected. When this is the case, the player jumps less then it should, netting,
            // at the end of the animation, a slightly negative Y than the beginning, which causes
            // problems. This is a workaround.
            //
            if start_y.is_none() {
                *start_y = Some(transform.translation.y);
            }

            if animation.current_frame < 1 {
                movement.y += 270. * time.delta_seconds();
            } else if animation.current_frame < 3 {
                movement.y -= 180. * time.delta_seconds();
            } else if animation.is_finished() {
                movement.y = start_y.unwrap();
                *start_y = None;
            }

            transform.translation += movement.extend(0.);
        }
    }
}

fn activate_hitbox(
    attack_query: Query<(Entity, &AttackFrames, &Parent), Without<Collider>>,
    fighter_query: Query<&Animation, With<State>>,
    mut commands: Commands,
) {
    for (entity, attack_frames, parent) in attack_query.iter() {
        if let Ok(animation) = fighter_query.get(parent.0) {
            if animation.current_frame >= attack_frames.startup
                && animation.current_frame <= attack_frames.active
            {
                //TODO: insert Collider based on size and transform offset in attack asset
                commands
                    .entity(entity)
                    .insert(Collider::cuboid(ATTACK_WIDTH * 0.8, ATTACK_HEIGHT * 0.8));
            }
        }
    }
}

fn deactivate_hitbox(
    query: Query<(Entity, &AttackFrames, &Parent), (With<Attack>, With<Collider>)>,
    fighter_query: Query<&Animation, With<State>>,
    mut commands: Commands,
) {
    for (entity, attack_frames, parent) in query.iter() {
        if let Ok(animation) = fighter_query.get(parent.0) {
            if animation.current_frame >= attack_frames.recovery {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn projectile_cleanup(
    query: Query<(Entity, &ProjectileLifetime), With<Attack>>,
    mut commands: Commands,
) {
    for (entity, timer) in query.iter() {
        if timer.0.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn projectile_tick(mut query: Query<&mut ProjectileLifetime, With<Attack>>, time: Res<Time>) {
    for mut timer in query.iter_mut() {
        timer.0.tick(time.delta());
    }
}
