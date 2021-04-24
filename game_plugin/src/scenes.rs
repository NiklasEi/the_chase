use crate::{GameStage, GameState};
use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;

pub enum CutScene {
    Intro,
}

pub struct ScenesPlugin;

impl Plugin for ScenesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::new()
                .with_run_criteria(should_run_a_scene.system())
                .with_system(run_scene.exclusive_system()),
        );
    }
}

fn should_run_a_scene(game_state: Res<GameState>) -> ShouldRun {
    if game_state.scene.is_some() {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn run_scene(_world: &mut World) {
    println!("running scene");
}
