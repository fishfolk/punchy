use std::collections::VecDeque;
use std::marker::PhantomData;

use bevy::{
    ecs::{
        entity::Entities,
        system::{CommandQueue, SystemParam},
    },
    prelude::*,
    reflect::FromType,
};
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    animation::{Animation, Facing},
    commands::DynamicEntityCommandsExt,
    enemy::Enemy,
    input::PlayerAction,
    metadata::FighterMeta,
    movement::Velocity,
    player::Player,
    GameState, Stats,
};

/// Plugin for managing fighter states
pub struct FighterStatePlugin;

#[derive(Clone, StageLabel)]
struct FighterStateFlushStage;

#[derive(Clone, SystemLabel)]
struct FighterStateCollectSystems;

impl Plugin for FighterStatePlugin {
    fn build(&self, app: &mut App) {
        app
            // // Debugging systems
            // .add_system_to_stage(CoreStage::First, || {
            //     info!("=====Start=====");
            // })
            // .add_system_to_stage(CoreStage::Last, || {
            //     info!("======End======");
            // })
            // State transition queue
            .init_resource::<StateTransitionCommandQueue>()
            // The collect systems
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                ConditionSet::new()
                    .label(FighterStateCollectSystems)
                    .run_in_state(GameState::InGame)
                    .with_system(collect_player_actions)
                    .with_system(collect_enemy_actions)
                    // .with_system(print_fighters)
                    .into(),
            )
            // The transition systems
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                ConditionSet::new()
                    .after(FighterStateCollectSystems)
                    .run_in_state(GameState::InGame)
                    .with_system(transition_from_idle)
                    .with_system(transition_from_flopping)
                    .into(),
            )
            // Flush stage
            .add_stage_before(
                CoreStage::Update,
                FighterStateFlushStage,
                SystemStage::single(flush_state_transition_commands.exclusive_system()),
            )
            // State handler systems
            .add_system_set_to_stage(
                CoreStage::Update,
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(idling)
                    .with_system(flopping)
                    .with_system(moving)
                    .into(),
            );
        // .add_system_to_stage(CoreStage::PostUpdate, print_fighters);
    }
}

/// Debugging system
fn _print_fighters(
    fighters: Query<
        (Entity, Option<&Idling>, Option<&Flopping>, Option<&Moving>),
        With<Handle<FighterMeta>>,
    >,
) {
    for (entity, idling, flopping, moving) in &fighters {
        info!("    {:?} {:?} {:?} {:?}", entity, idling, flopping, moving);
    }
}

/// Resource containing the state transition command queue
#[derive(Default, Deref, DerefMut)]
pub struct StateTransitionCommandQueue(CommandQueue);

/// System parameter very similar to the [`Commands`] parameter, but commands issued though it will
/// be flushed during [`FighterStateSystemSet::FlushStateTransitions`] instead of at the end of the
/// frame.
#[derive(SystemParam)]
pub struct StateTransitionCommands<'w, 's> {
    queue: ResMut<'w, StateTransitionCommandQueue>,
    entities: &'w Entities,
    #[system_param(ignore)]
    _phantom: PhantomData<&'s ()>,
}

impl<'w, 's> StateTransitionCommands<'w, 's> {
    pub fn commands<'a>(&'a mut self) -> Commands<'w, 'a> {
        Commands::new_from_entities(&mut self.queue, self.entities)
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

#[derive(Component, Default, Deref, DerefMut)]
pub struct StateTransitionIntents(VecDeque<StateTransition>);

//
// Fighter state components
//

#[derive(Component, Reflect, Default, Debug)]
#[component(storage = "SparseSet")]
pub struct Idling;
impl Idling {
    const PRIORITY: i32 = 0;
    const ANIMATION: &'static str = "idle";
}

#[derive(Component, Reflect, Default, Debug)]
#[component(storage = "SparseSet")]
pub struct Moving {
    velocity: Vec2,
}
impl Moving {
    const PRIORITY: i32 = 10;
    const ANIMATION: &'static str = "running";
}

#[derive(Component, Reflect, Default, Debug)]
#[component(storage = "SparseSet")]
pub struct Flopping {
    is_finished: bool,
}
impl Flopping {
    const PRIORITY: i32 = 20;
    const ANIMATION: &'static str = "attacking";
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
) {
    // info!("Collect");
    for (action_state, mut transition_intents, stats) in &mut players {
        if action_state.just_pressed(PlayerAction::FlopAttack) {
            transition_intents.push_back(StateTransition::new(
                Flopping::default(),
                Flopping::PRIORITY,
            ));
        }

        if action_state.pressed(PlayerAction::Move) {
            let dual_axis = action_state.clamped_axis_pair(PlayerAction::Move).unwrap();
            let direction = dual_axis.xy();

            transition_intents.push_back(StateTransition::new(
                Moving {
                    velocity: direction * stats.movement_speed,
                },
                Moving::PRIORITY,
            ));
        }
    }
}

// TODO: Implement AI actions
fn collect_enemy_actions(mut _enemies: Query<&mut StateTransitionIntents, With<Enemy>>) {}

//
// Transition states systems
//

/// Initiate any transitions from the idling state
fn transition_from_idle(
    mut transition_commands: StateTransitionCommands,
    mut fighters: Query<(Entity, &mut StateTransitionIntents), With<Idling>>,
) {
    // info!("trans from idle");
    let mut commands = transition_commands.commands();

    for (entity, mut transition_intents) in &mut fighters {
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
            }
        }
    }
}

// Initiate any transitions from the flopping state
fn transition_from_flopping(
    mut transition_commands: StateTransitionCommands,
    mut fighters: Query<(Entity, &mut StateTransitionIntents, &Flopping)>,
) {
    // info!("trans from flop");
    let mut commands = transition_commands.commands();

    for (entity, mut transition_intents, flopping) in &mut fighters {
        // Collect transitions and sort by priority
        let mut intents = transition_intents.drain(..).collect::<Vec<_>>();
        intents.sort_by(|a, b| a.priority.cmp(&b.priority));

        // For every intent
        for intent in intents {
            // If the intent is a higher priority than flopping
            if intent.priority > Flopping::PRIORITY {
                // Transition to the new state
                commands
                    .entity(entity)
                    .remove::<Flopping>()
                    .insert_dynamic(intent.reflect_component, intent.data);
                continue;
            }
        }

        // If we're done flopping
        if flopping.is_finished {
            // Go back to idle
            commands.entity(entity).remove::<Flopping>().insert(Idling);
        }
    }
}

//
// Flush state transitions system
//

fn flush_state_transition_commands(world: &mut World) {
    // info!("Flush");
    let mut queue = world
        .remove_resource::<StateTransitionCommandQueue>()
        .unwrap();

    queue.apply(world);

    world.insert_resource(queue);
}

//
// Handle state systems
//

/// Handle fighter idle state
fn idling(mut fighters: Query<(&mut Animation, &mut Velocity), With<Idling>>) {
    // info!("Idling");
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
fn flopping(mut fighters: Query<(&mut Animation, &mut Velocity, &mut Flopping)>) {
    // info!("Flopping");
    for (mut animation, mut velocity, mut flopping) in &mut fighters {
        // Make sure we stop moving ( this is temporary, we should do sort of a forward lurch )
        **velocity = Vec2::ZERO;

        // If we aren't playing the flop animation
        if animation.current_animation.as_deref() != Some(Flopping::ANIMATION) {
            // Start the flop animation from the beginning
            animation.play(Flopping::ANIMATION, false /* repeating */);

        // If the animation is done
        } else if animation.is_finished() {
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
    // info!("Moving");
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

        // Moving is a little different than the other states because we transition out of it to the
        // Idling state at the end of every frame, so that we only move if the player continually
        // inputs a movement.
        commands.entity(entity).remove::<Moving>().insert(Idling);
    }
}
