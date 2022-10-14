use std::{collections::VecDeque, time::Duration};

use bevy::{prelude::*, reflect::FromType, utils::HashSet};
use bevy_mod_js_scripting::ActiveScripts;
use bevy_rapier2d::prelude::{ActiveCollisionTypes, ActiveEvents, CollisionGroups, Sensor};
use iyes_loopless::prelude::*;
use leafwing_input_manager::{plugin::InputManagerSystem, prelude::ActionState};

use crate::{
    animation::{AnimatedSpriteSheetBundle, Animation, Facing, SyncFacing},
    attack::{Attack, Breakable},
    audio::AnimationAudioPlayback,
    collision::BodyLayers,
    consts,
    damage::{DamageEvent, Health},
    enemy::{Boss, Enemy},
    enemy_ai,
    fighter::{Attached, Inventory},
    input::PlayerAction,
    item::{Drop, Item, ItemBundle, Projectile, ScriptItemGrabEvent, ScriptItemThrowEvent},
    lifetime::Lifetime,
    metadata::{AttackMeta, AudioMeta, FighterMeta, ItemKind, ItemMeta, ItemSpawnMeta},
    movement::LinearVelocity,
    player::{AvailableAttacks, Player},
    Collider, GameState, Stats,
};

/// Plugin for managing fighter states
pub struct FighterStatePlugin;

/// The system set that fighter state change intents are collected
#[derive(Clone, SystemLabel)]
pub struct FighterStateCollectSystems;

impl Plugin for FighterStatePlugin {
    fn build(&self, app: &mut App) {
        app
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
                    .with_system(transition_from_flopping)
                    .with_system(transition_from_punching)
                    .with_system(transition_from_ground_slam)
                    .with_system(transition_from_knocked_back)
                    .with_system(transition_from_melee_attacking)
                    .with_system(transition_from_shooting)
                    .into(),
            )
            // State handler systems
            .add_system_set_to_stage(
                CoreStage::Update,
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(idling)
                    .with_system(flopping)
                    .with_system(punching)
                    .with_system(ground_slam)
                    .with_system(moving)
                    .with_system(throwing)
                    .with_system(grabbing)
                    .with_system(knocked_back)
                    .with_system(dying)
                    .with_system(holding)
                    .with_system(melee_attacking)
                    .with_system(shooting)
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
    pub fn apply<CurrentState: Component>(self, entity: Entity, commands: &mut Commands) -> bool {
        if !self.is_additive {
            commands.entity(entity).remove::<CurrentState>();
        }

        commands.add(move |world: &mut World| {
            // Insert the component stored in this state transition onto the entity
            self.reflect_component
                .insert(world, entity, self.data.as_reflect());
        });

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
        commands: &mut Commands,
    ) -> bool {
        // Collect transitions and sort by priority
        let mut transitions = self.drain(..).collect::<Vec<_>>();
        transitions.sort_by(|a, b| b.priority.cmp(&a.priority));

        // For every intent
        for intent in transitions {
            // If it's a higher priority
            if intent.priority > current_state_priority {
                // Apply the state
                let was_additive = intent.apply::<CurrentState>(entity, commands);

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
pub struct Flopping {
    /// The initial y-height of the figther when starting the attack
    pub start_y: f32,
    pub has_started: bool,
    pub is_finished: bool,
}
impl Flopping {
    pub const PRIORITY: i32 = 30;
    //TODO: return to change assets and this to "flopping"
    pub const ANIMATION: &'static str = "attacking";
}

/// Component indicating the player is punching
#[derive(Component, Reflect, Default, Debug)]
#[component(storage = "SparseSet")]
pub struct GroundSlam {
    /// The initial y-height of the figther when starting the attack
    pub start_y: f32,
    pub has_started: bool,
    pub is_finished: bool,
}
impl GroundSlam {
    pub const PRIORITY: i32 = 30;
    //TODO: return to change assets and this to "ground_slam"?
    pub const ANIMATION: &'static str = "attacking";
}

#[derive(Component, Reflect, Default, Debug)]
#[component(storage = "SparseSet")]
pub struct Punching {
    pub has_started: bool,
    pub is_finished: bool,
}
impl Punching {
    pub const PRIORITY: i32 = 30;
    pub const ANIMATION: &'static str = "attacking";
}

#[derive(Component, Reflect, Default, Debug)]
#[component(storage = "SparseSet")]
pub struct MeleeAttacking {
    pub has_started: bool,
    pub is_finished: bool,
}
impl MeleeAttacking {
    pub const PRIORITY: i32 = 30;
    pub const ANIMATION: &'static str = "slashing";
}

#[derive(Component, Reflect, Default, Debug)]
#[component(storage = "SparseSet")]
pub struct Shooting {
    pub has_started: bool,
    pub is_finished: bool,
    pub spawned_bullet: bool,
}
impl Shooting {
    pub const PRIORITY: i32 = 30;
    pub const ANIMATION: &'static str = "shooting";
}

/// Component indicating the player is holding a item on it's head
#[derive(Component, Reflect, Default, Debug)]
#[component(storage = "SparseSet")]
pub struct Holding {
    item: Handle<ItemMeta>,
}
impl Holding {
    pub const PRIORITY: i32 = 35;
}

/// Component indicating the player is getting knocked back
#[derive(Component, Reflect, Default, Debug)]
#[component(storage = "SparseSet")]
pub struct KnockedBack {
    pub velocity: Vec2,
    pub timer: Timer,
}
impl KnockedBack {
    pub const PRIORITY: i32 = 40;
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

#[derive(Component)]
pub struct BeingHeld;

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
            Option<&Holding>,
            &AvailableAttacks,
        ),
        With<Player>,
    >,
) {
    for (action_state, mut transition_intents, inventory, stats, holding, available_attacks) in
        &mut players
    {
        // Trigger attacks
        //TODO: can use flop attack again after input buffer/chaining
        if action_state.just_pressed(PlayerAction::Attack) && holding.is_none() {
            match available_attacks
                .0
                .last()
                .expect("Attack not loaded")
                .name
                .as_str()
            {
                "flop" => transition_intents.push_back(StateTransition::new(
                    Flopping::default(),
                    Flopping::PRIORITY,
                    false,
                )),
                "melee" => transition_intents.push_back(StateTransition::new(
                    MeleeAttacking::default(),
                    MeleeAttacking::PRIORITY,
                    false,
                )),
                "projectile" => transition_intents.push_back(StateTransition::new(
                    Shooting::default(),
                    Shooting::PRIORITY,
                    false,
                )),
                _ => {}
            }
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
                    velocity: direction * stats.movement_speed,
                },
                Moving::PRIORITY,
                false,
            ));
        }
    }
}

/// Look for attacks that have contacted a figher and knock them back
///
/// TODO: Not all attacks will have knockback. Maybe we should replace `damage_velocity` with
/// `damage_impulse` including the knockback time so that it can be ignored by this system if it's
/// velocity or time is zero.
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
                    //Knockback velocity feels strange right now
                    velocity: event.damage_velocity,
                    timer: Timer::from_seconds(0.50, false),
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
    mut commands: Commands,
    mut fighters: Query<(Entity, &mut StateTransitionIntents), With<Idling>>,
) {
    for (entity, mut transition_intents) in &mut fighters {
        // Transition to higher priority states
        transition_intents.transition_to_higher_priority_states::<Idling>(
            entity,
            Idling::PRIORITY,
            &mut commands,
        );
    }
}

// Initiate any transitions from the flopping state
fn transition_from_flopping(
    mut commands: Commands,
    mut fighters: Query<(Entity, &mut StateTransitionIntents, &Flopping)>,
) {
    'entity: for (entity, mut transition_intents, flopping) in &mut fighters {
        // Transition to any higher priority states
        let current_state_removed = transition_intents
            .transition_to_higher_priority_states::<Flopping>(
                entity,
                Flopping::PRIORITY,
                &mut commands,
            );

        // If our current state was removed, don't continue processing this fighter
        if current_state_removed {
            continue 'entity;
        }

        // If we're done flopping
        if flopping.is_finished {
            // Go back to idle
            commands.entity(entity).remove::<Flopping>().insert(Idling);
        }
    }
}

fn transition_from_punching(
    mut commands: Commands,
    mut fighters: Query<(Entity, &mut StateTransitionIntents, &Punching)>,
) {
    'entity: for (entity, mut transition_intents, punching) in &mut fighters {
        // Transition to any higher priority states
        let current_state_removed = transition_intents
            .transition_to_higher_priority_states::<Punching>(
                entity,
                Punching::PRIORITY,
                &mut commands,
            );

        // If our current state was removed, don't continue processing this fighter
        if current_state_removed {
            continue 'entity;
        }

        // If we're done attacking
        if punching.is_finished {
            // Go back to idle
            commands.entity(entity).remove::<Punching>().insert(Idling);
        }
    }
}

fn transition_from_ground_slam(
    mut commands: Commands,
    mut fighters: Query<(Entity, &mut StateTransitionIntents, &GroundSlam)>,
) {
    'entity: for (entity, mut transition_intents, ground_slam) in &mut fighters {
        // Transition to any higher priority states
        let current_state_removed = transition_intents
            .transition_to_higher_priority_states::<GroundSlam>(
                entity,
                GroundSlam::PRIORITY,
                &mut commands,
            );

        // If our current state was removed, don't continue processing this fighter
        if current_state_removed {
            continue 'entity;
        }

        // If we're done flopping
        if ground_slam.is_finished {
            // Go back to idle
            commands
                .entity(entity)
                .remove::<GroundSlam>()
                .insert(Idling);
        }
    }
}

// Initiate any transitions from the knocked back state
fn transition_from_knocked_back(
    mut commands: Commands,
    mut fighters: Query<(Entity, &mut StateTransitionIntents, &KnockedBack)>,
) {
    'entity: for (entity, mut transition_intents, knocked_back) in &mut fighters {
        // Transition to any higher priority states
        let current_state_removed = transition_intents
            .transition_to_higher_priority_states::<KnockedBack>(
                entity,
                KnockedBack::PRIORITY,
                &mut commands,
            );

        // If our current state was removed, don't continue processing this fighter
        if current_state_removed {
            continue 'entity;
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

fn transition_from_melee_attacking(
    mut commands: Commands,
    mut fighters: Query<(Entity, &mut StateTransitionIntents, &MeleeAttacking)>,
) {
    'entity: for (entity, mut transition_intents, melee_attacking) in &mut fighters {
        // Transition to any higher priority states
        let current_state_removed = transition_intents
            .transition_to_higher_priority_states::<MeleeAttacking>(
                entity,
                MeleeAttacking::PRIORITY,
                &mut commands,
            );

        // If our current state was removed, don't continue processing this fighter
        if current_state_removed {
            continue 'entity;
        }

        // If we're done attacking
        if melee_attacking.is_finished {
            // Go back to idle
            commands
                .entity(entity)
                .remove::<MeleeAttacking>()
                .insert(Idling);
        }
    }
}

fn transition_from_shooting(
    mut commands: Commands,
    mut fighters: Query<(Entity, &mut StateTransitionIntents, &Shooting)>,
) {
    'entity: for (entity, mut transition_intents, shooting) in &mut fighters {
        // Transition to any higher priority states
        let current_state_removed = transition_intents
            .transition_to_higher_priority_states::<Shooting>(
                entity,
                Shooting::PRIORITY,
                &mut commands,
            );

        // If our current state was removed, don't continue processing this fighter
        if current_state_removed {
            continue 'entity;
        }

        // If we're done attacking
        if shooting.is_finished {
            // Go back to idle
            commands.entity(entity).remove::<Shooting>().insert(Idling);
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
fn flopping(
    mut commands: Commands,
    mut fighters: Query<(
        Entity,
        &mut Animation,
        &mut Transform,
        &mut LinearVelocity,
        &Facing,
        &Handle<FighterMeta>,
        &mut Flopping,
        Option<&Player>,
        Option<&Enemy>,
    )>,
    fighter_assets: Res<Assets<FighterMeta>>,
) {
    for (
        entity,
        mut animation,
        mut transform,
        mut velocity,
        facing,
        meta_handle,
        mut flopping,
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

        if let Some(fighter) = fighter_assets.get(meta_handle) {
            // Start the attack
            if !flopping.has_started {
                flopping.has_started = true;
                flopping.start_y = transform.translation.y;

                // Start the attack  from the beginning
                animation.play(Flopping::ANIMATION, false);

                let mut offset = fighter.attack.hitbox.offset;
                if facing.is_left() {
                    offset.x *= -1.0
                }
                offset.y += fighter.collision_offset;
                let attack_frames = fighter.attack.frames;

                // Spawn the attack entity
                let attack_entity = commands
                    .spawn_bundle(TransformBundle::from_transform(
                        Transform::from_translation(offset.extend(0.0)),
                    ))
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
                            BodyLayers::ENEMY | BodyLayers::BREAKABLE_ITEM
                        } else {
                            BodyLayers::PLAYER
                        },
                    ))
                    .insert(Attack {
                        damage: fighter.attack.damage,
                        velocity: if facing.is_left() {
                            Vec2::NEG_X
                        } else {
                            Vec2::X
                        } * Vec2::new(consts::ATTACK_VELOCITY, 0.0),
                    })
                    .insert(attack_frames)
                    .id();
                commands.entity(entity).push_children(&[attack_entity]);

                // Play attack sound effect
                if let Some(effects) = fighter.audio.effect_handles.get(Flopping::ANIMATION) {
                    let fx_playback = AnimationAudioPlayback::new(
                        Flopping::ANIMATION.to_owned(),
                        effects.clone(),
                    );
                    commands.entity(entity).insert(fx_playback);
                }
            }

            // Reset velocity
            **velocity = Vec2::ZERO;

            // Do a forward jump thing
            //TODO: Fix hacky way to get a forward jump
            if animation.current_frame < fighter.attack.frames.recovery {
                if facing.is_left() {
                    velocity.x -= 200.0;
                } else {
                    velocity.x += 200.0;
                }
            }

            if animation.current_frame < fighter.attack.frames.startup {
                let v_per_frame = 200.0 / fighter.attack.frames.startup as f32;
                velocity.y += v_per_frame;
            } else if animation.current_frame < fighter.attack.frames.active {
                let v_per_frame =
                    200.0 / (fighter.attack.frames.active - fighter.attack.frames.startup) as f32;
                velocity.y -= v_per_frame;
            }

            if animation.is_finished() {
                // Stop moving
                **velocity = Vec2::ZERO;

                // Make sure we "land on the ground" ( i.e. the player y position hasn't changed )
                transform.translation.y = flopping.start_y;

                // Set flopping to finished
                flopping.is_finished = true;
            }
        }
    }
}

fn punching(
    mut commands: Commands,
    mut fighters: Query<(
        Entity,
        &mut Animation,
        &mut LinearVelocity,
        &Facing,
        &Handle<FighterMeta>,
        &mut Punching,
        Option<&Player>,
        Option<&Enemy>,
    )>,
    fighter_assets: Res<Assets<FighterMeta>>,
) {
    for (entity, mut animation, mut velocity, facing, meta_handle, mut punching, player, enemy) in
        &mut fighters
    {
        let is_player = player.is_some();
        let is_enemy = enemy.is_some();
        if !is_player && !is_enemy {
            // This system only knows how to attack for players and enemies
            continue;
        }

        if let Some(fighter) = fighter_assets.get(meta_handle) {
            if !punching.has_started {
                punching.has_started = true;

                // Start the attack  from the beginning
                animation.play(Punching::ANIMATION, false);

                let mut offset = fighter.attack.hitbox.offset;
                if facing.is_left() {
                    offset.x *= -1.0
                }
                offset.y += fighter.collision_offset;
                let attack_frames = fighter.attack.frames;
                // Spawn the attack entity
                let attack_entity = commands
                    .spawn_bundle(TransformBundle::from_transform(
                        Transform::from_translation(offset.extend(0.0)),
                    ))
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
                            BodyLayers::ENEMY | BodyLayers::BREAKABLE_ITEM
                        } else {
                            BodyLayers::PLAYER
                        },
                    ))
                    .insert(Attack {
                        damage: fighter.attack.damage,
                        velocity: if facing.is_left() {
                            Vec2::NEG_X
                        } else {
                            Vec2::X
                        } * Vec2::new(consts::ATTACK_VELOCITY, 0.0),
                    })
                    .insert(attack_frames)
                    .id();
                commands.entity(entity).push_children(&[attack_entity]);

                // Play attack sound effect
                if let Some(effects) = fighter.audio.effect_handles.get(Punching::ANIMATION) {
                    let fx_playback = AnimationAudioPlayback::new(
                        Punching::ANIMATION.to_owned(),
                        effects.clone(),
                    );
                    commands.entity(entity).insert(fx_playback);
                }
            }
        }

        **velocity = Vec2::ZERO;

        if animation.is_finished() {
            punching.is_finished = true;
        }
    }
}

/// The attacking state used for bosses
fn ground_slam(
    mut commands: Commands,
    mut fighters: Query<
        (
            Entity,
            &mut Animation,
            &mut Transform,
            &mut LinearVelocity,
            &Facing,
            &Handle<FighterMeta>,
            &mut GroundSlam,
        ),
        With<Boss>,
    >,
    fighter_assets: Res<Assets<FighterMeta>>,
) {
    for (
        entity,
        mut animation,
        mut transform,
        mut velocity,
        facing,
        meta_handle,
        mut ground_slam,
    ) in &mut fighters
    {
        // Start the attack
        if let Some(fighter) = fighter_assets.get(meta_handle) {
            let mut offset = fighter.attack.hitbox.offset;
            if facing.is_left() {
                offset.x *= -1.0
            }
            offset.y += fighter.collision_offset;
            let attack_frames = fighter.attack.frames;
            if !ground_slam.has_started {
                ground_slam.has_started = true;
                ground_slam.start_y = transform.translation.y;

                // Start the attack  from the beginning
                animation.play(GroundSlam::ANIMATION, false);

                // Spawn the attack entity
                let attack_entity = commands
                    .spawn_bundle(TransformBundle::from_transform(
                        Transform::from_translation(offset.extend(0.0)),
                    ))
                    .insert(Sensor)
                    .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC)
                    .insert(CollisionGroups::new(
                        BodyLayers::ENEMY_ATTACK,
                        BodyLayers::PLAYER,
                    ))
                    .insert(Attack {
                        damage: fighter.attack.damage,
                        velocity: if facing.is_left() {
                            Vec2::NEG_X
                        } else {
                            Vec2::X
                        } * Vec2::new(consts::ATTACK_VELOCITY, 0.0),
                    })
                    .insert(attack_frames)
                    .id();
                commands.entity(entity).push_children(&[attack_entity]);

                // Play attack sound effect
                if let Some(fighter) = fighter_assets.get(meta_handle) {
                    if let Some(effects) = fighter.audio.effect_handles.get(GroundSlam::ANIMATION) {
                        let fx_playback = AnimationAudioPlayback::new(
                            GroundSlam::ANIMATION.to_owned(),
                            effects.clone(),
                        );
                        commands.entity(entity).insert(fx_playback);
                    }
                }
            }

            // Reset velocity
            **velocity = Vec2::ZERO;

            if !animation.is_finished() {
                // Do a forward jump thing

                // Control x movement
                if animation.current_frame < attack_frames.startup {
                    if facing.is_left() {
                        velocity.x -= 50.0;
                    } else {
                        velocity.x += 50.0;
                    }
                }

                // Control y movement
                // TODO: Attack moves up and down the same amount, fixed distance, but it would be
                // nice to be able to tune the speed of the fall so it feels more impactful yet
                // doesnt have a "snap/reset effect" at the end of animation while still landing at
                // the same Y as started(?)
                // it might be nice to store movement properties as metadata attached to frame
                // ranges or individual frames?
                if animation.current_frame < attack_frames.startup {
                    let v_per_frame = 800.0 / attack_frames.startup as f32;
                    velocity.y += v_per_frame;
                } else if animation.current_frame < attack_frames.active {
                    let v_per_frame = 800.0 / (attack_frames.active - attack_frames.startup) as f32;
                    velocity.y -= v_per_frame;
                }

            // If the animation is finished
            } else {
                // Stop moving
                **velocity = Vec2::ZERO;

                // Make sure we "land on the ground" ( i.e. the player y position hasn't changed )
                transform.translation.y = ground_slam.start_y;

                // Set flopping to finished
                ground_slam.is_finished = true;
            }
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

        // When the animation is finished, despawn the fighter
        } else if animation.is_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Throw the item in the player's inventory
fn throwing(
    mut commands: Commands,
    mut fighters: Query<
        (
            Entity,
            &Transform,
            &Facing,
            &mut Inventory,
            Option<&mut AvailableAttacks>,
        ),
        With<Throwing>,
    >,
    being_held: Query<(Entity, &Parent), With<BeingHeld>>,
    weapon_held: Query<(Entity, &Parent), With<MeleeWeapon>>,
    pweapon_held: Query<(Entity, &Parent), With<ProjectileWeapon>>,
    mut items_assets: ResMut<Assets<ItemMeta>>,
    mut active_scripts: ResMut<ActiveScripts>,
    mut script_item_throw_events: ResMut<Events<ScriptItemThrowEvent>>,
) {
    for (entity, fighter_transform, facing, mut inventory, available_attacks) in &mut fighters {
        // If the player has an item in their inventory
        if let Some(item_meta) = inventory.take() {
            // Check what kind of item this is.
            //
            // TODO: We should probably create a flexible item system abstraction similar to the
            // fighter state abstraction so that items can flexibly defined without a
            // centralized enum.
            match &item_meta.kind {
                ItemKind::Throwable { .. } => {
                    // Throw the item!
                    commands.spawn_bundle(Projectile::from_thrown_item(
                        fighter_transform.translation + consts::THROW_ITEM_OFFSET.extend(0.0),
                        &item_meta,
                        facing,
                    ));
                }
                ItemKind::Script { script_handle, .. } => {
                    script_item_throw_events.send(ScriptItemThrowEvent {
                        fighter: entity,
                        script_handle: script_handle.clone_weak(),
                    });
                }
                ItemKind::BreakableBox {
                    ref item_handle, ..
                } => {
                    commands
                        .spawn_bundle(Projectile::from_thrown_item(
                            fighter_transform.translation + consts::THROW_ITEM_OFFSET.extend(0.0),
                            &item_meta,
                            facing,
                        ))
                        .insert(Drop {
                            item: items_assets
                                .get(item_handle)
                                .expect("Drop item not loaded!")
                                .clone(),
                        });

                    // Despawn head sprite
                    for (head_ent, parent) in being_held.iter() {
                        if parent.get() == entity {
                            commands.entity(head_ent).despawn_recursive();
                        }
                    }
                    commands.entity(entity).remove::<Holding>();
                }
                ItemKind::MeleeWeapon { .. } => {
                    //Drop item
                    let ground_offset = Vec3::new(0.0, consts::GROUND_Y, consts::ITEM_LAYER);

                    let item_spawn_meta = ItemSpawnMeta {
                        location: fighter_transform.translation - ground_offset,
                        item: String::new(),
                        item_handle: items_assets.add(item_meta.clone()),
                    };
                    let item_commands = commands.spawn_bundle(ItemBundle::new(&item_spawn_meta));
                    ItemBundle::spawn(
                        item_commands,
                        &item_spawn_meta,
                        &mut items_assets,
                        &mut active_scripts,
                    );

                    if let Some(mut available_attacks) = available_attacks {
                        available_attacks.0.pop();
                    }

                    // Despawn weapon sprite
                    for (weapon_ent, parent) in weapon_held.iter() {
                        if parent.get() == entity {
                            commands.entity(weapon_ent).despawn_recursive();
                        }
                    }
                }
                ItemKind::ProjectileWeapon { .. } => {
                    //Drop item
                    let ground_offset = Vec3::new(0.0, consts::GROUND_Y, consts::ITEM_LAYER);

                    let item_spawn_meta = ItemSpawnMeta {
                        location: fighter_transform.translation - ground_offset,
                        item: String::new(),
                        item_handle: items_assets.add(item_meta.clone()),
                    };
                    let item_commands = commands.spawn_bundle(ItemBundle::new(&item_spawn_meta));
                    ItemBundle::spawn(
                        item_commands,
                        &item_spawn_meta,
                        &mut items_assets,
                        &mut active_scripts,
                    );

                    if let Some(mut available_attacks) = available_attacks {
                        available_attacks.0.pop();
                    }

                    // Despawn weapon sprite
                    for (weapon_ent, parent) in pweapon_held.iter() {
                        if parent.get() == entity {
                            commands.entity(weapon_ent).despawn_recursive();
                        }
                    }
                }
            }
        }

        // Throwing is an "instant" state, that is removed at the end of every frame. Eventually it
        // will not be and will play a fighter animation.
        commands.entity(entity).remove::<Throwing>();
    }
}

// Trying to grab an item off the map
fn grabbing(
    mut commands: Commands,
    mut fighters: Query<
        (
            Entity,
            &Transform,
            &mut Inventory,
            &mut StateTransitionIntents,
            Option<&mut AvailableAttacks>,
        ),
        With<Grabbing>,
    >,
    items_query: Query<(Entity, &Transform, &Handle<ItemMeta>), With<Item>>,
    items_assets: Res<Assets<ItemMeta>>,
    mut script_item_grab_events: ResMut<Events<ScriptItemGrabEvent>>,
) {
    // We need to track the picked items, otherwise, in theory, two players could pick the same item.
    let mut picked_item_ids = HashSet::new();

    for (
        fighter_ent,
        fighter_transform,
        mut fighter_inventory,
        mut transition_intents,
        available_attacks,
    ) in &mut fighters
    {
        // If several items are at pick distance, an arbitrary one is picked.
        for (item_ent, item_transform, item) in &items_query {
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
                        match &items_assets.get(item).unwrap().kind {
                            ItemKind::Script { script_handle, .. } => {
                                script_item_grab_events.send(ScriptItemGrabEvent {
                                    fighter: fighter_ent,
                                    script_handle: script_handle.clone_weak(),
                                });
                                commands.entity(item_ent).despawn_recursive();
                            }
                            ItemKind::Throwable { damage: _ } => {
                                // If its throwable, pick up the item
                                picked_item_ids.insert(item_ent);
                                **fighter_inventory =
                                    Some(items_assets.get(item).expect("Item not loaded!").clone());
                                commands.entity(item_ent).despawn_recursive();
                            }
                            ItemKind::BreakableBox { .. } => {
                                // Transition to holding state
                                transition_intents.push_back(StateTransition::new(
                                    Holding { item: item.clone() },
                                    Holding::PRIORITY,
                                    true,
                                ));

                                picked_item_ids.insert(item_ent);
                                **fighter_inventory =
                                    Some(items_assets.get(item).expect("Item not loaded!").clone());
                                commands.entity(item_ent).despawn_recursive();
                            }
                            ItemKind::MeleeWeapon {
                                ref attack,
                                ref spritesheet,
                                ref audio,
                                ref sprite_offset,
                            } => {
                                // If its throwable, pick up the item
                                picked_item_ids.insert(item_ent);
                                **fighter_inventory =
                                    Some(items_assets.get(item).expect("Item not loaded!").clone());
                                commands.entity(item_ent).despawn_recursive();

                                if let Some(mut available_attacks) = available_attacks {
                                    available_attacks.0.push(attack.clone())
                                }

                                //Spawn weapon sprite on Player
                                let mut animated_sprite = AnimatedSpriteSheetBundle {
                                    sprite_sheet: SpriteSheetBundle {
                                        texture_atlas: spritesheet.atlas_handle[0].clone(),
                                        transform: Transform::from_xyz(
                                            sprite_offset.x,
                                            sprite_offset.y,
                                            0.,
                                        ),
                                        ..Default::default()
                                    },
                                    animation: Animation::new(
                                        spritesheet.animation_fps,
                                        spritesheet.animations.clone(),
                                    ),
                                };
                                animated_sprite.animation.current_animation =
                                    Some("idle".to_string());

                                let weapon = commands
                                    .spawn()
                                    .insert(MeleeWeapon {
                                        audio: audio.clone(),
                                        attack: attack.clone(),
                                    })
                                    .insert_bundle(animated_sprite)
                                    .insert(Attached {
                                        position_face: true,
                                    })
                                    .insert(Facing::default())
                                    .insert(SyncFacing)
                                    .id();
                                commands.entity(fighter_ent).add_child(weapon);
                            }
                            ItemKind::ProjectileWeapon {
                                ref attack,
                                ref spritesheet,
                                ref sprite_offset,
                                ref audio,
                                ref bullet_velocity,
                                ref bullet_lifetime,
                                ref ammo,
                                ref shoot_delay,
                            } => {
                                // If its throwable, pick up the item
                                picked_item_ids.insert(item_ent);
                                **fighter_inventory =
                                    Some(items_assets.get(item).expect("Item not loaded!").clone());
                                commands.entity(item_ent).despawn_recursive();

                                if let Some(mut available_attacks) = available_attacks {
                                    available_attacks.0.push(attack.clone())
                                }

                                //Spawn weapon sprite on Player
                                let mut animated_sprite = AnimatedSpriteSheetBundle {
                                    sprite_sheet: SpriteSheetBundle {
                                        texture_atlas: spritesheet.atlas_handle[0].clone(),
                                        transform: Transform::from_xyz(
                                            sprite_offset.x,
                                            sprite_offset.y,
                                            0.1,
                                        ),
                                        ..Default::default()
                                    },
                                    animation: Animation::new(
                                        spritesheet.animation_fps,
                                        spritesheet.animations.clone(),
                                    ),
                                };
                                animated_sprite.animation.current_animation =
                                    Some("idle".to_string());

                                let mut shoot_timer = Timer::from_seconds(*shoot_delay, false);
                                shoot_timer.set_elapsed(Duration::from_secs_f32(*shoot_delay));

                                let weapon = commands
                                    .spawn()
                                    .insert(ProjectileWeapon {
                                        attack: attack.clone(),
                                        animated_sprite: animated_sprite.clone(),
                                        audio: audio.clone(),
                                        bullet_velocity: *bullet_velocity,
                                        bullet_lifetime: *bullet_lifetime,
                                        ammo: *ammo,
                                        shoot_delay: shoot_timer,
                                    })
                                    .insert_bundle(animated_sprite)
                                    .insert(Attached {
                                        position_face: true,
                                    })
                                    .insert(Facing::default())
                                    .insert(SyncFacing)
                                    .id();
                                commands.entity(fighter_ent).add_child(weapon);
                            }
                        }
                    }
                    break;
                }
            }
        }
        // Grabbing is an "instant" state, that is removed at the end of every frame. Eventually it
        // may not be and it might play a fighter animation.
        commands.entity(fighter_ent).remove::<Grabbing>();
    }
}

/// Holding item
fn holding(
    mut commands: Commands,
    mut fighters: Query<(Entity, &Holding)>,
    being_held: Query<&Parent, With<BeingHeld>>,
    items_assets: Res<Assets<ItemMeta>>,
) {
    for (entity, holding) in &mut fighters {
        let mut already_holding = false;
        for parent in being_held.iter() {
            if parent.get() == entity {
                already_holding = true;
                break;
            }
        }

        if !already_holding {
            let image = items_assets
                .get(&holding.item)
                .expect("Item not loaded!")
                .clone()
                .image;

            let child = commands
                .spawn()
                .insert_bundle(SpriteBundle {
                    texture: image.image_handle.clone(),
                    transform: Transform::from_xyz(
                        0.,
                        consts::THROW_ITEM_OFFSET.y + image.image_size.y,
                        consts::PROJECTILE_Z,
                    ),
                    ..default()
                })
                .insert(BeingHeld)
                .insert(Facing::default())
                .insert(SyncFacing)
                .id();
            commands.entity(entity).add_child(child);
        }
    }
}

fn melee_attacking(
    mut commands: Commands,
    mut fighters: Query<
        (
            Entity,
            Option<&mut MeleeAttacking>,
            Option<&Player>,
            Option<&Enemy>,
            &AvailableAttacks,
            &mut LinearVelocity,
            &Facing,
        ),
        Without<MeleeWeapon>,
    >,
    mut melee_weapons: Query<(Entity, &Parent, &mut Animation, &MeleeWeapon)>,
) {
    for (entity, melee_attack, player, enemy, available_attacks, mut velocity, facing) in
        &mut fighters
    {
        let is_player = player.is_some();
        let is_enemy = enemy.is_some();
        if !is_player && !is_enemy {
            // This system only knows how to attack for players and enemies
            continue;
        }

        let mut melee_weapon = None;
        for (weapon_ent, parent, animation, weapon) in &mut melee_weapons {
            if parent.get() == entity {
                melee_weapon = Some((animation, weapon.audio.clone(), weapon_ent));
            }
        }

        if let Some((mut animation, audio, weapon_ent)) = melee_weapon {
            //Check if it's attacking
            if let Some(mut melee_attack) = melee_attack {
                if !melee_attack.has_started {
                    melee_attack.has_started = true;

                    // Start the attack from the beginning
                    animation.play("slashing", false);

                    let attack = available_attacks.0.last().expect("Attack not loaded");

                    let offset = attack.hitbox.offset;
                    let attack_frames = attack.frames;
                    // Spawn the attack entity
                    let attack_entity = commands
                        .spawn_bundle(TransformBundle::from_transform(
                            Transform::from_translation(offset.extend(0.0)),
                        ))
                        .insert(Sensor)
                        .insert(ActiveEvents::COLLISION_EVENTS)
                        .insert(
                            ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC,
                        )
                        .insert(CollisionGroups::new(
                            if is_player {
                                BodyLayers::PLAYER_ATTACK
                            } else {
                                BodyLayers::ENEMY_ATTACK
                            },
                            if is_player {
                                BodyLayers::ENEMY | BodyLayers::BREAKABLE_ITEM
                            } else {
                                BodyLayers::PLAYER
                            },
                        ))
                        .insert(Attack {
                            damage: attack.damage,
                            velocity: if facing.is_left() {
                                Vec2::NEG_X
                            } else {
                                Vec2::X
                            } * Vec2::new(consts::ATTACK_VELOCITY, 0.0),
                        })
                        .insert(attack_frames)
                        .id();
                    commands.entity(weapon_ent).push_children(&[attack_entity]);

                    // Play attack sound effect
                    if let Some(effects) = audio.effect_handles.get(MeleeAttacking::ANIMATION) {
                        let fx_playback = AnimationAudioPlayback::new(
                            MeleeAttacking::ANIMATION.to_owned(),
                            effects.clone(),
                        );
                        commands.entity(weapon_ent).insert(fx_playback);
                    }
                }
                **velocity = Vec2::ZERO;

                if animation.is_finished() {
                    melee_attack.is_finished = true;
                }
            }
        }
    }
}

fn shooting(
    mut commands: Commands,
    mut fighters: Query<
        (
            Entity,
            Option<&mut Shooting>,
            Option<&Player>,
            Option<&Enemy>,
            &AvailableAttacks,
            &mut LinearVelocity,
            &Facing,
        ),
        Without<ProjectileWeapon>,
    >,
    mut projectile_weapons: Query<(
        Entity,
        &Parent,
        &mut Animation,
        &mut ProjectileWeapon,
        &GlobalTransform,
    )>,
    shooting_particles: Query<(&Animation, Entity, &Particle), Without<ProjectileWeapon>>,
    time: Res<Time>,
) {
    for (entity, shooting, player, enemy, available_attacks, mut velocity, facing) in &mut fighters
    {
        let is_player = player.is_some();
        let is_enemy = enemy.is_some();
        if !is_player && !is_enemy {
            // This system only knows how to attack for players and enemies
            continue;
        }

        let mut projectile_weapon = None;
        for (weapon_ent, parent, animation, weapon, weapon_gtransform) in &mut projectile_weapons {
            if parent.get() == entity {
                projectile_weapon = Some((animation, weapon_ent, weapon_gtransform, weapon));
            }
        }

        if let Some((mut animation, weapon_ent, weapon_gtransform, mut weapon)) = projectile_weapon
        {
            //Tick shoot delay
            weapon.shoot_delay.tick(time.delta());

            //Check if it's attacking
            if let Some(mut shooting) = shooting {
                let attack = available_attacks.0.last().expect("Attack not loaded");

                if !shooting.has_started && weapon.ammo > 0 && weapon.shoot_delay.finished() {
                    shooting.has_started = true;
                    weapon.shoot_delay.reset();

                    // Start the attack from the beginning
                    animation.play("shooting", false);

                    //Add particles
                    let mut animated_sprite = weapon.animated_sprite.clone();
                    animated_sprite.sprite_sheet.transform = Transform::from_xyz(0., 0., 0.1);
                    animated_sprite.sprite_sheet.sprite.flip_x = facing.is_left();
                    animated_sprite.animation.play("shooting_particles", false);

                    let weapon_particles = commands
                        .spawn()
                        .insert_bundle(animated_sprite.clone())
                        .insert(Particle)
                        .id();
                    commands.entity(weapon_ent).add_child(weapon_particles);

                    //Sound
                    if let Some(effects) = weapon.audio.effect_handles.get(Shooting::ANIMATION) {
                        let fx_playback = AnimationAudioPlayback::new(
                            Shooting::ANIMATION.to_owned(),
                            effects.clone(),
                        );
                        commands.entity(weapon_ent).insert(fx_playback);
                    }
                }

                if animation.current_animation == Some("shooting".to_string())
                    && animation.current_frame == attack.frames.startup
                    && !shooting.spawned_bullet
                {
                    //Spawn bullet
                    shooting.spawned_bullet = true;
                    weapon.ammo -= 1;

                    let direction_mul = if facing.is_left() {
                        Vec2::new(-1.0, 1.0)
                    } else {
                        Vec2::ONE
                    };

                    let mut animated_sprite = weapon.animated_sprite.clone();
                    animated_sprite.animation.play("bullet", false);
                    animated_sprite.sprite_sheet.transform = Transform::from_xyz(
                        weapon_gtransform.translation().x,
                        weapon_gtransform.translation().y,
                        consts::PROJECTILE_Z,
                    );

                    let bullet_attack = commands
                        .spawn_bundle(TransformBundle::from_transform(
                            Transform::from_translation(
                                (attack.hitbox.offset * direction_mul).extend(0.0),
                            ),
                        ))
                        .insert(Sensor)
                        .insert(ActiveEvents::COLLISION_EVENTS)
                        .insert(
                            ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC,
                        )
                        .insert(CollisionGroups::new(
                            BodyLayers::PLAYER_ATTACK,
                            BodyLayers::ENEMY | BodyLayers::BREAKABLE_ITEM,
                        ))
                        .insert(Attack {
                            damage: attack.damage,
                            velocity: Vec2::new(consts::ATTACK_VELOCITY, 0.0) * direction_mul,
                        })
                        .insert(Breakable::new(0, true))
                        .insert(Collider::cuboid(
                            attack.hitbox.size.x / 2.,
                            attack.hitbox.size.y / 2.,
                        ))
                        .id();

                    commands
                        .spawn_bundle(animated_sprite)
                        .insert(Lifetime(Timer::from_seconds(weapon.bullet_lifetime, false)))
                        .insert(LinearVelocity(
                            Vec2::new(weapon.bullet_velocity, 0.) * direction_mul,
                        ))
                        .add_child(bullet_attack);
                }

                **velocity = Vec2::ZERO;

                if animation.is_finished() {
                    shooting.is_finished = true;
                    animation.play("idle", false);
                }
            }
        }
    }

    //Check if particle is done
    for (animation, particle_ent, _) in shooting_particles.iter() {
        if animation.is_finished() {
            commands.entity(particle_ent).despawn_recursive();
        }
    }
}

#[derive(Component)]
pub struct MeleeWeapon {
    pub audio: AudioMeta,
    pub attack: AttackMeta,
}

#[derive(Component)]
pub struct ProjectileWeapon {
    pub audio: AudioMeta,
    pub attack: AttackMeta,
    pub animated_sprite: AnimatedSpriteSheetBundle,
    pub ammo: usize,
    pub bullet_velocity: f32,
    pub bullet_lifetime: f32,
    pub shoot_delay: Timer,
}

#[derive(Component)]
pub struct Particle;
