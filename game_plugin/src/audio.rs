use crate::actions::Actions;
use crate::loading::AudioAssets;
use crate::GameStage;
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioChannel, AudioPlugin, AudioSource};

pub struct InternalAudioPlugin;

impl Plugin for InternalAudioPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(AudioChannels {
            effects: AudioChannel::new("effects".to_owned()),
        })
        .add_plugin(AudioPlugin)
        .add_event::<AudioEffect>()
        .add_system_set(SystemSet::on_enter(GameStage::Playing).with_system(start_audio.system()))
        .add_system_set(SystemSet::on_update(GameStage::Playing).with_system(play_effect.system()))
        .add_system_set(SystemSet::on_exit(GameStage::Playing).with_system(stop_audio.system()));
    }
}

pub struct AudioEffect {
    pub handle: Handle<AudioSource>,
}

struct AudioChannels {
    effects: AudioChannel,
}

fn start_audio(audio: Res<Audio>, channels: Res<AudioChannels>) {
    audio.set_volume_in_channel(0.3, &channels.effects);
}

fn stop_audio(audio: Res<Audio>, channels: Res<AudioChannels>) {
    audio.stop_channel(&channels.effects);
}

fn play_effect(
    mut events: EventReader<AudioEffect>,
    audio: Res<Audio>,
    channels: Res<AudioChannels>,
) {
    for effect in events.iter() {
        audio.play_in_channel(effect.handle.clone(), &channels.effects)
    }
}
