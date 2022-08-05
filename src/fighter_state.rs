use std::collections::VecDeque;

use bevy::{prelude::*, reflect::FromType, utils::HashSet};
use bevy_rapier2d::prelude::{ActiveCollisionTypes, ActiveEvents, CollisionGroups, Sensor};
use iyes_loopless::prelude::*;
use leafwing_input_manager::{plugin::InputManagerSystem, prelude::ActionState};

use crate::{
    animation::{Animation, Facing},
    attack::{Attack, AttackFrames},
    audio::AnimationAudioPlayback,
    collision::BodyLayers,
    commands::{
        flush_custom_commands, CustomCommands, DynamicEntityCommandsExt, InitCustomCommandsAppExt,
    },
    consts,
    damage::{DamageEvent, Health},
    enemy::Enemy,
    enemy_ai,
    fighter::Inventory,
    input::PlayerAction,
    item::{Item, Projectile},
    metadata::{FighterMeta, ItemMeta},
    movement::LinearVelocity,
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
                    .after(InputManagerSystem::Update)
                    .run_in_state(GameState::InGame)
                    .with_system(collect_fighter_eliminations)
                    .with_system(collect_attack_knockbacks)
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
                    .with_system(attacking)
                    .with_system(moving)
                    .with_system(throwing)
                    .with_system(grabbing)
                    .with_system(knocked_back)
                    .with_system(dying)
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
    /// If a state transition is additive, it means that the existing state should not be removed
    /// when this state is applied.
    is_additive: bool,
}

impl StateTransition {
    /// Create a new fighter state event from the given state and priority
    pub fn new<T>(component: T, priority: i32, is_additive: bool) -> Self
    where
        T: Reflect + Default + Component,
    {
        let reflect_component = <ReflectComponent as FromType<T>>::from_type();
        let data = Box::new(component) as _;
        Self {
            reflect_component,
            data,
            priority,
            is_additive,
        }
    }

    /// Apply this state transition to the given entity.
    ///
    /// Returns whether or not the transition was additive.
    ///
    /// If a transition was additive, it means the current state will still be active.
    pub fn apply<CurrentState: Component>(
        self,
        entity: Entity,
        transition_commands: &mut CustomCommands<TransitionCmds>,
    ) -> bool {
        let mut commands = transition_commands.commands();
        let mut entity_cmds = commands.entity(entity);

        if !self.is_additive {
            entity_cmds.remove::<CurrentState>();
        }

        entity_cmds.insert_dynamic(self.reflect_component, self.data);

        self.is_additive
    }
}

/// Component on fighters that contains the queue of state transition intents
#[derive(Component, Default, Deref, DerefMut)]
pub struct StateTransitionIntents(VecDeque<StateTransition>);

impl StateTransitionIntents {
    /// Helper to transition to any higher priority states
    ///
    /// Returns `true` if a non-additive state has been transitioned to and the current state has been
    /// removed.
    fn transition_to_higher_priority_states<CurrentState: Component>(
        &mut self,
        entity: Entity,
        current_state_priority: i32,
        transition_commands: &mut CustomCommands<TransitionCmds>,
    ) -> bool {
        // Collect transitions and sort by priority
        let mut transitions = self.drain(..).collect::<Vec<_>>();
        transitions.sort_by(|a, b| b.priority.cmp(&a.priority));

        // For every intent
        for intent in transitions {
            // If it's a higher priority
            if intent.priority > current_state_priority {
                // Apply the state
                let was_additive = intent.apply::<CurrentState>(entity, transition_commands);

                // If it was not an additive transition
                if !was_additive {
                    // Skip processing other transitions because our current state was removed, and
                    // return true to indicate that a non-additive transition has been performed.
                    return true;
                }
            }
        }

        // I we got here we are still in the same state so return false to indicate no non-additive
        // transitions have been performed.
        false
    }
}

/// Component on fighters containing the action intent queue.
///
/// Actions are things that the fighter should try to do that doesn't represent a state change. The
/// current fighter state must choose to act upon an action intent for it to be effect, and it will
/// otherwise be ignored.q
#[derive(Component, Default, Deref, DerefMut)]
pub struct ActionIntents(VecDeque<Box<dyn Reflect>>);

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

/// The player is throwing an item
#[derive(Component, Reflect, Default, Debug)]
pub struct Throwing;
impl Throwing {
    pub const PRIORITY: i32 = 15;
}

/// The player is grabbing an item ( or trying to)
#[derive(Component, Reflect, Default, Debug)]
pub struct Grabbing;
impl Grabbing {
    pub const PRIORITY: i32 = Throwing::PRIORITY;
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

/// Component indicating the player is dying
#[derive(Component, Reflect, Default, Debug)]
#[component(storage = "SparseSet")]
pub struct Dying;
impl Dying {
    pub const PRIORITY: i32 = 1000;
    pub const ANIMATION: &'static str = "dying";
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
            &Inventory,
            &Stats,
        ),
        With<Player>,
    >,
    time: Res<Time>,
) {
    for (action_state, mut transition_intents, inventory, stats) in &mut players {
        // Trigger attacks
        if action_state.just_pressed(PlayerAction::Attack) {
            transition_intents.push_back(StateTransition::new(
                Attacking::default(),
                Attacking::PRIORITY,
                false,
            ));
        }

        // Trigger grab/throw
        if action_state.just_pressed(PlayerAction::Throw) {
            if inventory.is_some() {
                transition_intents.push_back(StateTransition::new(
                    Throwing,
                    Throwing::PRIORITY,
                    true,
                ));
            } else {
                transition_intents.push_back(StateTransition::new(
                    Grabbing,
                    Grabbing::PRIORITY,
                    true,
                ));
            }
        }

        // Trigger movement
        if action_state.pressed(PlayerAction::Move) {
            let dual_axis = action_state.clamped_axis_pair(PlayerAction::Move).unwrap();
            let direction = dual_axis.xy();

            transition_intents.push_back(StateTransition::new(
                Moving {
                    velocity: direction * stats.movement_speed * time.delta_seconds(),
                },
                Moving::PRIORITY,
                false,
            ));
        }
    }
}

/// Look for attacks that have contacted a figher and knock them back
fn collect_attack_knockbacks(
    mut fighters: Query<&mut StateTransitionIntents, With<Handle<FighterMeta>>>,
    mut damage_events: EventReader<DamageEvent>,
) {
    for event in damage_events.iter() {
        // If the damaged entity was a fighter
        if let Ok(mut transition_intents) = fighters.get_mut(event.damaged_entity) {
            // Trigger knock back
            transition_intents.push_back(StateTransition::new(
                KnockedBack {
                    velocity: event.damage_velocity,
                    timer: Timer::from_seconds(0.18, false),
                },
                KnockedBack::PRIORITY,
                false,
            ));
        }
    }
}

/// Look for fighters with their health depleated and transition them to dying state
fn collect_fighter_eliminations(
    mut fighters: Query<(&Health, &mut StateTransitionIntents), With<Handle<FighterMeta>>>,
) {
    for (health, mut transition_intents) in &mut fighters {
        // If the fighter health is depleted
        if **health <= 0 {
            // Transition to dying state
            transition_intents.push_back(StateTransition::new(Dying, Dying::PRIORITY, false));
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
    for (entity, mut transition_intents) in &mut fighters {
        // Transition to higher priority states
        transition_intents.transition_to_higher_priority_states::<Idling>(
            entity,
            Idling::PRIORITY,
            &mut transition_commands,
        );
    }
}

// Initiate any transitions from the flopping state
fn transition_from_attacking(
    mut transition_commands: CustomCommands<TransitionCmds>,
    mut fighters: Query<(Entity, &mut StateTransitionIntents, &Attacking)>,
) {
    'entity: for (entity, mut transition_intents, flopping) in &mut fighters {
        // Transition to any higher priority states
        let current_state_removed = transition_intents
            .transition_to_higher_priority_states::<Attacking>(
                entity,
                Attacking::PRIORITY,
                &mut transition_commands,
            );

        // If our current state was removed, don't continue processing this fighter
        if current_state_removed {
            continue 'entity;
        }

        // If we're done flopping
        if flopping.is_finished {
            // Go back to idle
            transition_commands
                .commands()
                .entity(entity)
                .remove::<Attacking>()
                .insert(Idling);
        }
    }
}

// Initiate any transitions from the knocked back state
fn transition_from_knocked_back(
    mut transition_commands: CustomCommands<TransitionCmds>,
    mut fighters: Query<(Entity, &mut StateTransitionIntents, &KnockedBack)>,
) {
    'entity: for (entity, mut transition_intents, knocked_back) in &mut fighters {
        // Transition to any higher priority states
        let current_state_removed = transition_intents
            .transition_to_higher_priority_states::<KnockedBack>(
                entity,
                KnockedBack::PRIORITY,
                &mut transition_commands,
            );

        // If our current state was removed, don't continue processing this fighter
        if current_state_removed {
            continue 'entity;
        }

        // Transition to idle when finished
        if knocked_back.timer.finished() {
            transition_commands
                .commands()
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
fn idling(mut fighters: Query<(&mut Animation, &mut LinearVelocity), With<Idling>>) {
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

/// Handle fighter attacking state
///
/// > **Note:** This system currently applies attacks for both enemies and players, doing a sort of
/// > jumping "punch". In the future there will be different attacks, which will each have their own
/// > state system, and we will trigger different attack states for different players and enemies,
/// > based on the attacks available to that fighter.
fn attacking(
    mut commands: Commands,
    mut fighters: Query<(
        Entity,
        &mut Animation,
        &mut LinearVelocity,
        &Facing,
        &Stats,
        &Handle<FighterMeta>,
        &mut Attacking,
        Option<&Player>,
        Option<&Enemy>,
    )>,
    fighter_assets: Res<Assets<FighterMeta>>,
    time: Res<Time>,
) {
    for (
        entity,
        mut animation,
        mut velocity,
        facing,
        stats,
        meta_handle,
        mut attacking,
        player,
        enemy,
    ) in &mut fighters
    {
        let is_player = player.is_some();
        let is_enemy = enemy.is_some();
        if !is_player && !is_enemy {
            // This system only knows how to attack for players and enemies
            continue;
        }

        // Start the attack
        if !attacking.has_started {
            attacking.has_started = true;

            // Start the attack  from the beginning
            animation.play(Attacking::ANIMATION, false);

            // Spawn the attack entity
            let attack_entity = commands
                .spawn_bundle(TransformBundle::default())
                .insert(Sensor)
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC)
                .insert(CollisionGroups::new(
                    if is_player {
                        BodyLayers::PLAYER_ATTACK
                    } else {
                        BodyLayers::ENEMY_ATTACK
                    },
                    if is_player {
                        BodyLayers::ENEMY
                    } else {
                        BodyLayers::PLAYER
                    },
                ))
                .insert(Attack {
                    damage: stats.damage,
                    velocity: if facing.is_left() {
                        Vec2::NEG_X
                    } else {
                        Vec2::X
                    } * Vec2::new(consts::ATTACK_VELOCITY, 0.0),
                })
                // TODO: Read from figher metadata
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

        // Reset velocity
        **velocity = Vec2::ZERO;

        // Do a forward jump thing
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
            attacking.is_finished = true;
        }
    }
}

/// Handle fighter moving state
fn moving(
    mut commands: Commands,
    mut fighters: Query<(
        Entity,
        &mut Animation,
        &mut Facing,
        &mut LinearVelocity,
        &Moving,
    )>,
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

/// Update knocked back players
fn knocked_back(
    mut fighters: Query<(
        &mut Animation,
        &Facing,
        &mut LinearVelocity,
        &mut KnockedBack,
    )>,
    time: Res<Time>,
) {
    for (mut animation, facing, mut velocity, mut knocked_back) in &mut fighters {
        // If this is the start of the knock back
        if knocked_back.timer.elapsed_secs() == 0.0 {
            // Calculate animation to use based on attack direction
            let is_left = knocked_back.velocity.x < 0.0;
            let use_left_anim = if facing.is_left() { !is_left } else { is_left };
            let animation_name = if use_left_anim {
                KnockedBack::ANIMATION_LEFT
            } else {
                KnockedBack::ANIMATION_RIGHT
            };

            // Play the animation
            animation.play(animation_name, false);
        }

        // Tick the knock-back timer
        knocked_back.timer.tick(time.delta());

        // Set our figher velocity to the knock back velocity
        **velocity = knocked_back.velocity;
    }
}

/// Update dying players
fn dying(
    mut commands: Commands,
    mut fighters: Query<(Entity, &mut Animation, &mut LinearVelocity), With<Dying>>,
) {
    for (entity, mut animation, mut velocity) in &mut fighters {
        // Start playing the dying animation if it isn't already
        if animation.current_animation.as_deref() != Some(Dying::ANIMATION) {
            **velocity = Vec2::ZERO;
            animation.play(Dying::ANIMATION, false);

        // When the animatino is finished, despawn the fighter
        } else if animation.is_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Throw the item in the player's inventory
fn throwing(
    mut commands: Commands,
    mut fighters: Query<(Entity, &Transform, &Facing, &mut Inventory), With<Throwing>>,
    item_assets: Res<Assets<ItemMeta>>,
) {
    for (entity, fighter_transform, facing, mut inventory) in &mut fighters {
        // If the player has an item in their inventory
        if let Some(meta_handle) = inventory.take() {
            // If the item asset has loaded
            if let Some(item) = item_assets.get(&meta_handle) {
                // Throw the item!
                commands.spawn_bundle(Projectile::from_thrown_item(
                    fighter_transform.translation + consts::THROW_ITEM_OFFSET.extend(0.0),
                    item,
                    facing,
                ));

            // If the item asset isn't loaded yet
            } else {
                // This shouldn't happen because we make sure our assets are loaded before we start
                // the game. But just in case, we'll want to know about it if it does happen.
                warn!("Thrown item not spawned because it has not been loaded yet");
            }
        }

        // Throwing is an "instant" state, that is removed at the end of every frame
        commands.entity(entity).remove::<Throwing>();
    }
}

// Trying to grab an item off the map
fn grabbing(
    mut commands: Commands,
    mut fighters: Query<(Entity, &Transform, &mut Inventory), With<Grabbing>>,
    items_query: Query<(Entity, &Transform, &Handle<ItemMeta>), With<Item>>,
) {
    // We need to track the picked items, otherwise, in theory, two players could pick the same item.
    let mut picked_item_ids = HashSet::new();

    for (fighter_ent, fighter_transform, mut fighter_inventory) in &mut fighters {
        // If several items are at pick distance, an arbitrary one is picked.
        for (item_ent, item_transform, item_meta_handle) in &items_query {
            if !picked_item_ids.contains(&item_ent) {
                // Get the distance the figher is from the item
                let fighter_item_distance = fighter_transform
                    .translation
                    .truncate()
                    .distance(item_transform.translation.truncate());

                // If we are close enough
                if fighter_item_distance <= consts::PICK_ITEM_RADIUS {
                    // And our fighter isn't carrying another item
                    if fighter_inventory.is_none() {
                        // Pick up the item
                        picked_item_ids.insert(item_ent);
                        **fighter_inventory = Some(item_meta_handle.clone());
                        commands.entity(item_ent).despawn_recursive();
                    }

                    break;
                }
            }
        }

        // Throwing is an "instant" state, that is removed at the end of every frame
        commands.entity(fighter_ent).remove::<Grabbing>();
    }
}
