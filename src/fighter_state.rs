use std::collections::VecDeque;

use bevy::{prelude::*, reflect::FromType};
use bevy_rapier2d::prelude::{ActiveCollisionTypes, ActiveEvents, CollisionGroups, Sensor};
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    animation::{Animation, Facing},
    attack::{Attack, AttackFrames},
    audio::AnimationAudioPlayback,
    collisions::BodyLayers,
    commands::{
        flush_custom_commands, CustomCommands, DynamicEntityCommandsExt, InitCustomCommandsAppExt,
    },
    enemy_ai,
    input::PlayerAction,
    metadata::FighterMeta,
    movement::Velocity,
    player::Player,
    GameState, Stats,
};

/// Plugin for managing fighter states
pub struct FighterStatePlugin;

/// The system set that fighter state change intents are collected
#[derive(Clone, SystemLabel)]
pub struct FighterStateCollectSystems;

/// [`CustomCommands`] marker type.
pub struct TransitionCmds;

impl Plugin for FighterStatePlugin {
    fn build(&self, app: &mut App) {
        app
            // State transition queue
            .init_custom_commands::<TransitionCmds>()
            // The collect systems
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                ConditionSet::new()
                    .label(FighterStateCollectSystems)
                    .run_in_state(GameState::InGame)
                    .with_system(collect_player_actions)
                    .with_system(
                        enemy_ai::set_target_near_player.chain(enemy_ai::emit_enemy_intents),
                    )
                    .into(),
            )
            // The transition systems
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                ConditionSet::new()
                    .after(FighterStateCollectSystems)
                    .run_in_state(GameState::InGame)
                    .with_system(transition_from_idle)
                    .with_system(transition_from_attacking)
                    .with_system(transition_from_knocked_back)
                    .into(),
            )
            // Flush stage
            .add_system_to_stage(
                CoreStage::PreUpdate,
                flush_custom_commands::<TransitionCmds>
                    .exclusive_system()
                    .at_end(),
            )
            // State handler systems
            .add_system_set_to_stage(
                CoreStage::Update,
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(idling)
                    .with_system(flopping)
                    .with_system(moving)
                    .with_system(knocked_back)
                    .into(),
            );
    }
}

/// A state transition
pub struct StateTransition {
    /// The [`ReflectComponent`] of the state component we want to transition to
    reflect_component: ReflectComponent,
    /// The data of the component we want to transition to
    data: Box<dyn Reflect>,
    /// The priority of the state transition
    ///
    /// A priority of `i32::MAX` should usually be transitioned to immediately regardless of
    /// the current state.
    priority: i32,
}

impl StateTransition {
    /// Create a new fighter state event from the given state and priority
    pub fn new<T>(component: T, priority: i32) -> Self
    where
        T: Reflect + Default + Component,
    {
        let reflect_component = <ReflectComponent as FromType<T>>::from_type();
        let data = Box::new(component) as _;
        Self {
            reflect_component,
            data,
            priority,
        }
    }
}

/// Component on fighters that contains the queue of state transition intents
#[derive(Component, Default, Deref, DerefMut)]
pub struct StateTransitionIntents(VecDeque<StateTransition>);

//
// Fighter state components
//

/// Component indicating the player is idling
#[derive(Component, Reflect, Default, Debug)]
#[component(storage = "SparseSet")]
pub struct Idling;
impl Idling {
    pub const PRIORITY: i32 = 0;
    pub const ANIMATION: &'static str = "idle";
}

/// Component indicating the player is moving
#[derive(Component, Reflect, Default, Debug)]
#[component(storage = "SparseSet")]
pub struct Moving {
    pub velocity: Vec2,
}
impl Moving {
    pub const PRIORITY: i32 = 10;
    pub const ANIMATION: &'static str = "running";
}

/// Component indicating the player is flopping
#[derive(Component, Reflect, Default, Debug)]
#[component(storage = "SparseSet")]
pub struct Attacking {
    pub has_started: bool,
    pub is_finished: bool,
}
impl Attacking {
    pub const PRIORITY: i32 = 30;
    pub const ANIMATION: &'static str = "attacking";
}

/// Component indicating the player is getting knocked back
#[derive(Component, Reflect, Default, Debug)]
#[component(storage = "SparseSet")]
pub struct KnockedBack {
    pub velocity: Vec2,
    pub timer: Timer,
}
impl KnockedBack {
    pub const PRIORITY: i32 = 20;
    pub const ANIMATION_LEFT: &'static str = "knocked_left";
    pub const ANIMATION_RIGHT: &'static str = "knocked_right";
}

//
// Fighter input collector systems
//

/// Emits state transitions based on fighter actions
fn collect_player_actions(
    mut players: Query<
        (
            &ActionState<PlayerAction>,
            &mut StateTransitionIntents,
            &Stats,
        ),
        With<Player>,
    >,
    time: Res<Time>,
) {
    for (action_state, mut transition_intents, stats) in &mut players {
        if action_state.pressed(PlayerAction::FlopAttack) {
            transition_intents.push_back(StateTransition::new(
                Attacking::default(),
                Attacking::PRIORITY,
            ));
        }

        // For testing only: knock back player when they hit the throw button.
        if action_state.pressed(PlayerAction::Throw) {
            transition_intents.push_back(StateTransition::new(
                KnockedBack {
                    velocity: Vec2::new(-200.0, 0.0),
                    timer: Timer::from_seconds(0.15, false),
                },
                KnockedBack::PRIORITY,
            ));
        }

        if action_state.pressed(PlayerAction::Move) {
            let dual_axis = action_state.clamped_axis_pair(PlayerAction::Move).unwrap();
            let direction = dual_axis.xy();

            transition_intents.push_back(StateTransition::new(
                Moving {
                    velocity: direction * stats.movement_speed * time.delta_seconds(),
                },
                Moving::PRIORITY,
            ));
        }
    }
}

//
// Transition states systems
//

/// Initiate any transitions from the idling state
fn transition_from_idle(
    mut transition_commands: CustomCommands<TransitionCmds>,
    mut fighters: Query<(Entity, &mut StateTransitionIntents), With<Idling>>,
) {
    let mut commands = transition_commands.commands();

    'entity: for (entity, mut transition_intents) in &mut fighters {
        // Collect transitions and sort by priority
        let mut transitions = transition_intents.drain(..).collect::<Vec<_>>();
        transitions.sort_by(|a, b| a.priority.cmp(&b.priority));

        // Since idling is the lowest priority state, just transition to the highest priority in the
        // intent list.
        //
        // This logic may become more sophisticated later.
        if let Some(transition) = transitions.pop() {
            if transition.priority > Idling::PRIORITY {
                commands
                    .entity(entity)
                    .remove::<Idling>()
                    .insert_dynamic(transition.reflect_component, transition.data);
                continue 'entity;
            }
        }
    }
}

// Initiate any transitions from the flopping state
fn transition_from_attacking(
    mut transition_commands: CustomCommands<TransitionCmds>,
    mut fighters: Query<(Entity, &mut StateTransitionIntents, &Attacking)>,
) {
    let mut commands = transition_commands.commands();

    'entity: for (entity, mut transition_intents, flopping) in &mut fighters {
        // Collect transitions and sort by priority
        let mut intents = transition_intents.drain(..).collect::<Vec<_>>();
        intents.sort_by(|a, b| a.priority.cmp(&b.priority));

        // For every intent
        for intent in intents {
            // If the intent is a higher priority than flopping
            if intent.priority > Attacking::PRIORITY {
                // Transition to the new state
                commands
                    .entity(entity)
                    .remove::<Attacking>()
                    .insert_dynamic(intent.reflect_component, intent.data);
                continue 'entity;
            }
        }

        // If we're done flopping
        if flopping.is_finished {
            // Go back to idle
            commands.entity(entity).remove::<Attacking>().insert(Idling);
        }
    }
}

// Initiate any transitions from the knocked back state
fn transition_from_knocked_back(
    mut transition_commands: CustomCommands<TransitionCmds>,
    mut fighters: Query<(Entity, &mut StateTransitionIntents, &KnockedBack)>,
) {
    let mut commands = transition_commands.commands();

    'entity: for (entity, mut transition_intents, knocked_back) in &mut fighters {
        // Collect transitions and sort by priority
        let mut intents = transition_intents.drain(..).collect::<Vec<_>>();
        intents.sort_by(|a, b| a.priority.cmp(&b.priority));

        for intent in intents {
            // Transition to higher priority intents
            if intent.priority > KnockedBack::PRIORITY {
                commands
                    .entity(entity)
                    .remove::<KnockedBack>()
                    .insert_dynamic(intent.reflect_component, intent.data);
                continue 'entity;
            }
        }

        // Transition to idle when finished
        if knocked_back.timer.finished() {
            commands
                .entity(entity)
                .remove::<KnockedBack>()
                .insert(Idling);
        }
    }
}

//
// Handle state systems
//

/// Handle fighter idle state
fn idling(mut fighters: Query<(&mut Animation, &mut Velocity), With<Idling>>) {
    for (mut animation, mut velocity) in &mut fighters {
        // If we aren't playing the idle animation
        if animation.current_animation.as_deref() != Some(Idling::ANIMATION) {
            // Start the idle animation from the beginning
            animation.play(Idling::ANIMATION, true /* repeating */)
        }

        // Stop moving playe when we idle
        **velocity = Vec2::ZERO;
    }
}

/// Handle fighter flopping state
fn flopping(
    mut commands: Commands,
    mut fighters: Query<(
        Entity,
        &mut Animation,
        &mut Velocity,
        &Facing,
        &Stats,
        &Handle<FighterMeta>,
        &mut Attacking,
    )>,
    fighter_assets: Res<Assets<FighterMeta>>,
    time: Res<Time>,
) {
    for (entity, mut animation, mut velocity, facing, stats, meta_handle, mut flopping) in
        &mut fighters
    {
        // Spawn the flop attack
        if !flopping.has_started {
            flopping.has_started = true;

            // Start the flop animation from the beginning
            animation.play(Attacking::ANIMATION, false /* repeating */);

            // Spawn the attack entity
            let attack_entity = commands
                .spawn_bundle(TransformBundle::default())
                .insert(Sensor)
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC)
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

            // Play attack sound effect
            if let Some(fighter) = fighter_assets.get(meta_handle) {
                if let Some(effects) = fighter.audio.effect_handles.get(Attacking::ANIMATION) {
                    let fx_playback = AnimationAudioPlayback::new(
                        Attacking::ANIMATION.to_owned(),
                        effects.clone(),
                    );
                    commands.entity(entity).insert(fx_playback);
                }
            }
        }

        **velocity = Vec2::ZERO;

        //TODO: Fix hacky way to get a forward jump
        if animation.current_frame < 3 {
            let dt = time.delta_seconds();

            if facing.is_left() {
                velocity.x -= 20_000.0 * dt;
            } else {
                velocity.x += 20_000.0 * dt;
            }

            if animation.current_frame < 1 {
                velocity.y += 18_000. * dt;
            } else if animation.current_frame < 3 {
                velocity.y -= 9_000. * dt;
            }
        }

        // If the animation is done
        if animation.is_finished() {
            // Set flopping to finished
            flopping.is_finished = true;
        }
    }
}

/// Handle fighter moving state
fn moving(
    mut commands: Commands,
    mut fighters: Query<(Entity, &mut Animation, &mut Facing, &mut Velocity, &Moving)>,
) {
    for (entity, mut animation, mut facing, mut velocity, moving) in &mut fighters {
        // If we aren't playing the moving animation
        if animation.current_animation.as_deref() != Some(Moving::ANIMATION) {
            // Start the moving animation from the beginning
            animation.play(Moving::ANIMATION, true /* repeating */);
        }

        // Update our velocity to match our movement velocity
        **velocity = moving.velocity;

        // Make sure we face in the direction we are moving
        if velocity.x > 0.0 {
            *facing = Facing::Right
        } else if velocity.x < 0.0 {
            *facing = Facing::Left
        }

        // Moving is a little different than the other states because we transition out of it at the
        // end of every frame, so that we only move if the player continually inputs a movement.
        commands.entity(entity).remove::<Moving>().insert(Idling);
    }
}

fn knocked_back(
    mut fighters: Query<(&mut Animation, &Facing, &mut Velocity, &mut KnockedBack)>,
    time: Res<Time>,
) {
    for (mut animation, facing, mut velocity, mut knocked_back) in &mut fighters {
        // If this is the start of the knock back
        if knocked_back.timer.elapsed_secs() == 0.0 {
            let is_left = knocked_back.velocity.x < 0.0;
            let use_left_anim = if facing.is_left() { !is_left } else { is_left };
            let animation_name = if use_left_anim {
                KnockedBack::ANIMATION_LEFT
            } else {
                KnockedBack::ANIMATION_RIGHT
            };

            animation.play(animation_name, false);
        }

        knocked_back.timer.tick(time.delta());

        **velocity = knocked_back.velocity;
    }
}
