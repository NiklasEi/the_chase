mod paths;

use crate::loading::paths::PATHS;
use crate::GameStage;
use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy_kira_audio::AudioSource;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_enter(GameStage::Loading).with_system(start_loading.system()),
        )
        .add_system_set(SystemSet::on_update(GameStage::Loading).with_system(check_state.system()));
    }
}

pub struct LoadingState {
    textures: Vec<HandleUntyped>,
    fonts: Vec<HandleUntyped>,
    audio: Vec<HandleUntyped>,
    maps: Vec<HandleUntyped>,
}

pub struct FontAssets {
    pub fira_sans: Handle<Font>,
}

pub struct AudioAssets {
    pub fall: Handle<AudioSource>,
}

pub struct TextureAssets {
    pub texture_player: Handle<Texture>,
}

fn start_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut fonts: Vec<HandleUntyped> = vec![];
    fonts.push(asset_server.load_untyped(PATHS.fira_sans));

    let mut audio: Vec<HandleUntyped> = vec![];
    audio.push(asset_server.load_untyped(PATHS.audio_fall));

    let texture_names = [
        "box",
        "dirtexit",
        "dirtpath",
        "dirtwall",
        "groundexit",
        "groundpath",
        "groundwall",
        "lavaexit",
        "lavapath",
        "lavawall",
        "stoneexit",
        "stonepath",
        "stonewall",
    ];
    let mut textures: Vec<HandleUntyped> = vec![];
    textures.push(asset_server.load_untyped(PATHS.texture_player));
    for name in &texture_names {
        textures.push(asset_server.load_untyped(&format!("textures/{}.png", name)[..]));
    }

    let mut maps: Vec<HandleUntyped> = vec![];
    maps.push(asset_server.load_untyped("map/ground.tmx"));
    maps.push(asset_server.load_untyped("map/dirt.tmx"));
    maps.push(asset_server.load_untyped("map/stone.tmx"));

    commands.insert_resource(LoadingState {
        textures,
        fonts,
        audio,
        maps,
    });
}

fn check_state(
    mut commands: Commands,
    mut state: ResMut<State<GameStage>>,
    asset_server: Res<AssetServer>,
    loading_state: Res<LoadingState>,
) {
    if LoadState::Loaded
        != asset_server.get_group_load_state(loading_state.fonts.iter().map(|handle| handle.id))
    {
        return;
    }
    if LoadState::Loaded
        != asset_server.get_group_load_state(loading_state.textures.iter().map(|handle| handle.id))
    {
        return;
    }
    if LoadState::Loaded
        != asset_server.get_group_load_state(loading_state.audio.iter().map(|handle| handle.id))
    {
        return;
    }
    if LoadState::Loaded
        != asset_server.get_group_load_state(loading_state.maps.iter().map(|handle| handle.id))
    {
        return;
    }

    commands.insert_resource(FontAssets {
        fira_sans: asset_server.get_handle(PATHS.fira_sans),
    });

    commands.insert_resource(AudioAssets {
        fall: asset_server.get_handle(PATHS.audio_fall),
    });

    commands.insert_resource(TextureAssets {
        texture_player: asset_server.get_handle(PATHS.texture_player),
    });

    state.set(GameStage::Menu).unwrap();
}
