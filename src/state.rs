use bevy::prelude::{App, Component, Plugin, Query};

use crate::animation::Animation;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(exit_knocked_state);
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

fn exit_knocked_state(mut query: Query<(&mut State, &Animation)>) {
    for (mut state, animation) in query.iter_mut() {
        if (*state == State::KnockedLeft || *state == State::KnockedRight)
            && animation.is_finished()
        {
            *state = State::Idle;
        }
    }
}
