// Multiple sounds can be played by one channel, but splitting music/effects is cleaner.
// Also for cleanness (named channels have evident function), we don't use the default channel.

use bevy::{prelude::*, utils::HashMap};
use bevy_kira_audio::{AudioChannel, AudioSource};

use crate::{
    animation::Animation,
    metadata::{GameMeta, LevelMeta},
    state::State,
};

/// For readability.
const IMPOSSIBLE_ANIMATION_I: usize = usize::MAX;

pub struct MusicChannel;

pub struct EffectsChannel;

pub fn set_audio_channels_volume(
    music_channel: Res<AudioChannel<MusicChannel>>,
    effects_channel: Res<AudioChannel<EffectsChannel>>,
) {
    music_channel.set_volume(0.5);
    effects_channel.set_volume(0.5);
}

/// Add this to a fighter, when want to play sound effects attached to certain animation indexes.
#[derive(Component)]
pub struct FighterStateEffectsPlayback {
    pub state: State,
    pub effects: HashMap<usize, Handle<AudioSource>>,
    pub last_played: Option<usize>,
}

impl FighterStateEffectsPlayback {
    pub fn new(state: State, effects: HashMap<usize, Handle<AudioSource>>) -> Self {
        Self {
            state,
            effects,
            last_played: None,
        }
    }
}

pub fn fighter_sound_effect(
    mut commands: Commands,
    mut query: Query<(Entity, &State, &Animation, &mut FighterStateEffectsPlayback)>,
    effects_channel: Res<AudioChannel<EffectsChannel>>,
) {
    for (entity, fighter_state, animation, mut state_effects) in query.iter_mut() {
        // The safest way to remove the sound component is on the next state, because the component
        // can be remove only at the last frame of animation, which in theory, may be skipped if
        // there is an unexpected lag.
        // Alternatively, we could just not care, since subsequent states+effects will overwrite
        // the component.
        if *fighter_state != state_effects.state {
            commands
                .entity(entity)
                .remove::<FighterStateEffectsPlayback>();

            continue;
        }

        if let Some(fighter_animation_i) = animation.get_current_index() {
            if let Some(audio_handle) = state_effects.effects.get(&fighter_animation_i) {
                if state_effects.last_played.unwrap_or(IMPOSSIBLE_ANIMATION_I)
                    != fighter_animation_i
                {
                    effects_channel.play(audio_handle.clone());
                    state_effects.last_played = Some(fighter_animation_i);
                }
            }
        }
    }
}

// For currently unclear reasons, this system runs twice even if it's added via
// `add_enter_system(GameState::MainMenu`, so we need to track the playback state.
pub fn play_menu_music(
    mut playing: Local<bool>,
    game_meta: Res<GameMeta>,
    music_channel: Res<AudioChannel<MusicChannel>>,
) {
    if !*playing {
        music_channel.play(game_meta.main_menu.music_handle.clone());
        *playing = true;
    }
}

pub fn stop_menu_music(music_channel: Res<AudioChannel<MusicChannel>>) {
    music_channel.stop();
}

pub fn play_level_music(
    level_handle: Res<Handle<LevelMeta>>,
    assets: Res<Assets<LevelMeta>>,
    music_channel: Res<AudioChannel<MusicChannel>>,
) {
    if let Some(level) = assets.get(level_handle.clone_weak()) {
        music_channel.play(level.music_handle.clone());
    }
}

pub fn stop_level_music(music_channel: Res<AudioChannel<MusicChannel>>) {
    music_channel.stop();
}
