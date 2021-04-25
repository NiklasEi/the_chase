mod actions;
mod audio;
mod loading;
mod map;
mod menu;
mod player;
mod scenes;

use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::player::PlayerPlugin;

use bevy::app::AppBuilder;
// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use crate::map::MapPlugin;
use crate::scenes::{CutScene, ScenesPlugin};
use anyhow::Result;
use bevy::asset::{AssetLoader, AssetServerSettings, BoxedFuture, LoadContext, LoadedAsset};
use bevy::prelude::*;
use bevy_reflect::TypeUuid;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use tiled::Map;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameStage {
    Loading,
    Playing,
    Menu,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        let asset_folder = app
            .world()
            .get_resource::<AssetServerSettings>()
            .unwrap()
            .asset_folder
            .clone();

        app.add_asset::<TiledMap>()
            .add_asset_loader(TiledMapLoader::new(asset_folder))
            .add_state(GameStage::Loading)
            .init_resource::<GameState>()
            .add_plugin(LoadingPlugin)
            .add_plugin(ActionsPlugin)
            .add_plugin(ScenesPlugin)
            .add_plugin(MenuPlugin)
            .add_plugin(InternalAudioPlugin)
            .add_plugin(PlayerPlugin)
            .add_plugin(MapPlugin)
            // .add_plugin(FrameTimeDiagnosticsPlugin::default())
            // .add_plugin(LogDiagnosticsPlugin::default())
            ;
    }
}

pub struct GameState {
    pub frozen: bool,
    pub scene: Option<CutScene>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            frozen: false,
            scene: None,
        }
    }
}

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "6a9fc4ca-b5a5-94d6-613c-522e2d9fe86d"]
pub struct TiledMap {
    map: Map,
}

pub struct TiledMapLoader {
    asset_folder: PathBuf,
}

impl TiledMapLoader {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        TiledMapLoader {
            asset_folder: path.as_ref().to_path_buf(),
        }
    }
}

impl AssetLoader for TiledMapLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            let path = load_context.path();
            #[cfg(not(target_arch = "wasm32"))]
            let root_dir = bevy::asset::FileAssetIo::get_root_path();
            #[cfg(target_arch = "wasm32")]
            let root_dir = PathBuf::from("");

            let map = tiled::parse_with_path(
                BufReader::new(bytes),
                &root_dir.join(&self.asset_folder.as_path().join(path)),
            )
            .expect("Failed to parse map");
            load_context.set_default_asset(LoadedAsset::new(TiledMap { map }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["tmx"]
    }
}
