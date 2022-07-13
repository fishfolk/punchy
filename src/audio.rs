// Multiple sounds can be played by one channel, but splitting music/effects is cleaner.
// Also for cleanness (named channels have evident function), we don't use the default channel.

use bevy::prelude::Res;
use bevy_kira_audio::AudioChannel;

pub struct MusicChannel;

pub struct EffectsChannel;

pub fn set_audio_channels_volume(
    music_channel: Res<AudioChannel<MusicChannel>>,
    effects_channel: Res<AudioChannel<EffectsChannel>>,
) {
    music_channel.set_volume(0.5);
    effects_channel.set_volume(0.5);
}
