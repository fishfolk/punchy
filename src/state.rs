use bevy::prelude::{App, Component, CoreStage, Plugin, Query, Without};

use crate::{animation::Animation, movement::Knockback};

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PreUpdate, exit_knocked_state);
    }
}

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum State {
    Idle,
    Running,
    Attacking,
    KnockedLeft,
    KnockedRight,
    Dying,
}

impl State {
    pub fn set(&mut self, state: State) {
        *self = state;
    }

    pub fn is_knocked(&self) -> bool {
        match self {
            State::KnockedLeft | State::KnockedRight => true,
            _ => false,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        State::Idle
    }
}

fn exit_knocked_state(mut query: Query<(&mut State, &Animation), Without<Knockback>>) {
    for (mut state, animation) in query.iter_mut() {
        if state.is_knocked() && animation.is_finished() {
            state.set(State::Idle);
        }
    }
}
