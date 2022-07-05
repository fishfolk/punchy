use std::ops::Range;

use crate::{state::State, GameStage, GameState};
use bevy::{
    core::{Time, Timer},
    prelude::{App, Changed, Component, CoreStage, Plugin, Query, Res},
    sprite::TextureAtlasSprite,
    utils::HashMap,
};
use iyes_loopless::condition::ConditionSet;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            GameStage::Animation,
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(animate_on_state_changed)
                .with_system(animation_flipping)
                .with_system(animation_cycling)
                .into(),
        );
    }
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
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

#[derive(serde::Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Clip {
    pub frames: Range<usize>,
    #[serde(default)]
    pub repeat: bool,
}

#[derive(Component)]
pub struct Animation {
    pub animations: HashMap<State, Clip>,
    pub current_frame: usize,
    current_state: Option<State>,
    pub timer: Timer,
    pub played_once: bool,
}

impl Animation {
    pub fn new(fps: f32, animations: HashMap<State, Clip>) -> Self {
        Self {
            animations,
            current_frame: 0,
            current_state: None,
            timer: Timer::from_seconds(fps, false),
            played_once: false,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.played_once
    }

    pub fn is_repeating(&self) -> bool {
        if let Some(state) = self.current_state {
            if let Some(clip) = self.animations.get(&state) {
                return clip.repeat;
            }
        }

        false
    }

    pub fn is_last_frame(&self) -> bool {
        if let Some(indices) = self.get_current_indices() {
            if let Some(index) = self.get_current_index() {
                return index >= indices.end;
            }
        }

        false
    }

    pub fn get_current_indices(&self) -> Option<&Range<usize>> {
        if let Some(state) = self.current_state {
            match self.animations.get(&state) {
                Some(clip) => return Some(&clip.frames),
                None => return None,
            }
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

fn animate_on_state_changed(mut query: Query<(&mut Animation, &State), Changed<State>>) {
    for (mut animation, state) in query.iter_mut() {
        if animation.current_state != Some(*state) {
            animation.played_once = false;
            animation.current_frame = 0;
            animation.current_state = Some(*state);
            animation.timer.reset();
        }
    }
}

fn animation_cycling(mut query: Query<(&mut TextureAtlasSprite, &mut Animation)>, time: Res<Time>) {
    //TODO: Add a tick method on Animation
    for (mut texture_atlas_sprite, mut animation) in query.iter_mut() {
        if animation.is_finished() && !animation.is_repeating() {
            return;
        }

        animation.timer.tick(time.delta());

        if animation.timer.finished() {
            animation.timer.reset();

            if animation.is_last_frame() {
                animation.played_once = true; // Check if animation player here because we need to wait the last frame

                if animation.is_repeating() {
                    animation.current_frame = 0;
                }
            } else {
                animation.current_frame += 1;
            }
        }

        if let Some(index) = animation.get_current_index() {
            texture_atlas_sprite.index = index;
        }
    }
}

fn animation_flipping(mut query: Query<(&mut TextureAtlasSprite, &Facing)>) {
    for (mut texture_atlas_sprite, facing) in query.iter_mut() {
        texture_atlas_sprite.flip_x = facing.is_left();
    }
}
