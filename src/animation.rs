use std::ops::Range;

use bevy::{
    core::{Time, Timer},
    prelude::{Component, Query, Res},
    sprite::TextureAtlasSprite,
    utils::HashMap,
};

use crate::{state::State, Player};

#[derive(Component)]
pub struct Animation {
    pub animations: HashMap<State, Range<usize>>,
    pub current_frame: usize,
    pub current_state: Option<State>,
    pub timer: Timer,
    pub played_once: bool,
}

#[derive(Component, PartialEq, Eq, Clone)]
pub enum Facing {
    Left,
    Right,
}

impl Facing {
    pub fn is_left(&self) -> bool {
        self == &Facing::Left
    }

    pub fn set(&mut self, facing: Facing) {
        *self = facing;
    }
}

impl Animation {
    pub fn new(fps: f32, animations: HashMap<State, Range<usize>>) -> Self {
        Self {
            animations,
            current_frame: 0,
            current_state: None,
            timer: Timer::from_seconds(fps, false),
            played_once: false,
        }
    }

    pub fn set(&mut self, state: State) {
        if self.current_state == Some(state) {
            return;
        }

        self.played_once = false;
        self.current_frame = 0;
        self.current_state = Some(state);
        self.timer.reset();
    }

    pub fn is_finished(&self) -> bool {
        self.played_once
    }

    pub fn is_last_frame(&self) -> bool {
        if let Some(indices) = self.get_current_indices() {
            if let Some(index) = self.get_current_index() {
                return index >= indices.end;
            }
        }

        return false;
    }

    pub fn get_current_indices(&self) -> Option<&Range<usize>> {
        if let Some(state) = self.current_state {
            return self.animations.get(&state);
        }

        None
    }

    pub fn get_current_index(&self) -> Option<usize> {
        if let Some(indices) = self.get_current_indices() {
            return Some(indices.start + self.current_frame);
        }

        None
    }
}

pub fn animation_cycling(
    mut query: Query<(&mut TextureAtlasSprite, &mut Animation)>,
    time: Res<Time>,
) {
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

pub fn animation_flipping(mut query: Query<(&mut TextureAtlasSprite, &Facing)>) {
    for (mut texture_atlas_sprite, facing) in query.iter_mut() {
        texture_atlas_sprite.flip_x = facing.is_left();
    }
}

//TODO: Switch this to a genreic state machine
pub fn player_animation_state(mut query: Query<(&Player, &mut Animation)>) {
    let (player, mut animation) = query.single_mut();

    animation.set(player.state);
}
