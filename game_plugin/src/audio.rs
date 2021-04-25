use crate::GameStage;
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioChannel, AudioPlugin, AudioSource};
use crate::loading::AudioAssets;

pub struct InternalAudioPlugin;

impl Plugin for InternalAudioPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(AudioChannels {
            effects: AudioChannel::new("effects".to_owned()),
            background: AudioChannel::new("background".to_owned()),
        })
        .add_plugin(AudioPlugin)
            .add_event::<AudioEffect>()
            .add_event::<BackgroundAudio>()
            .add_event::<ResumeBackground>()
            .add_event::<PauseBackground>()
        .add_system_set(SystemSet::on_enter(GameStage::Menu).with_system(start_audio.system()))
        .add_system_set(SystemSet::new()
            .with_system(play_effect.system())
            .with_system(play_background.system())
            .with_system(resume_background.system())
            .with_system(pause_background.system()))
        .add_system_set(SystemSet::on_exit(GameStage::Playing).with_system(stop_audio.system()));
    }
}

pub struct PauseBackground;
pub struct ResumeBackground;

pub struct BackgroundAudio {
    pub handle: Handle<AudioSource>,
}

pub struct AudioEffect {
    pub handle: Handle<AudioSource>,
}

struct AudioChannels {
    effects: AudioChannel,
    background: AudioChannel
}

fn start_audio(audio: Res<Audio>, channels: Res<AudioChannels>) {
    audio.set_volume_in_channel(0.3, &channels.effects);
    audio.set_volume_in_channel(0.2, &channels.background);
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

fn play_background(
    mut events: EventReader<BackgroundAudio>,
    audio: Res<Audio>,
    channels: Res<AudioChannels>,
) {
    for background in events.iter() {
        audio.stop_channel(&channels.background);
        audio.play_looped_in_channel(background.handle.clone(), &channels.background);
    }
}

fn resume_background(
    mut events: EventReader<ResumeBackground>,
    audio: Res<Audio>,
    channels: Res<AudioChannels>,
) {
    for _event in events.iter() {
        audio.resume_channel(&channels.background);
    }
}

fn pause_background(
    mut events: EventReader<PauseBackground>,
    audio: Res<Audio>,
    channels: Res<AudioChannels>,
) {
    for _event in events.iter() {
        audio.pause_channel(&channels.background);
    }
}
