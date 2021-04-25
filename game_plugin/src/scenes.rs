use crate::map::Map;
use crate::player::{calc_camera_position, Player, PlayerCamera};
use crate::{GameStage, GameState};
use bevy::ecs::schedule::ShouldRun;
use bevy::ecs::world::WorldCell;
use bevy::prelude::*;
use bevy::render::camera::OrthographicProjection;
use std::time::Duration;

pub struct ScenesPlugin;

pub struct TriggerScene {
    pub scene: CutScene,
}

#[derive(Clone)]
pub enum CutScene {
    Intro { from: (f32, f32), to: (f32, f32) },
    MapTransition { to: Map },
}

impl Plugin for ScenesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<TriggerScene>().add_system_set(
            SystemSet::on_update(GameStage::Playing)
                .with_system(run_intro.system())
                .with_system(run_transition_scene.system())
                .with_system(trigger_scene.system()),
        );
    }
}

fn run_intro(
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
    mut camera: Query<&mut Transform, With<PlayerCamera>>,
) {
    if let Some(scene) = &game_state.scene {
        if let CutScene::Intro { from, to } = scene {
            const CAMERA_ON_PLAYER: Duration = Duration::from_millis(500);
            const CAMERA_TO_GOAL: Duration = Duration::from_millis(1500);
            const CAMERA_ON_GOAL: Duration = Duration::from_secs(2);
            const CAMERA_BACK_TO_PLAYER: Duration = Duration::from_secs(3);

            if time
                .time_since_startup()
                .lt(&(game_state.scene_start + CAMERA_ON_PLAYER))
            {
                return;
            }

            if time
                .time_since_startup()
                .gt(&(game_state.scene_start + CAMERA_BACK_TO_PLAYER))
            {
                if let Ok(mut transform) = camera.single_mut() {
                    transform.translation.x = from.0;
                    transform.translation.y = from.1;
                }
                game_state.scene = None;
                game_state.frozen = false;
                return;
            }

            if time
                .time_since_startup()
                .gt(&(game_state.scene_start + CAMERA_TO_GOAL))
                && time
                    .time_since_startup()
                    .lt(&(game_state.scene_start + CAMERA_ON_GOAL))
            {
                if let Ok(mut transform) = camera.single_mut() {
                    transform.translation.x = to.0;
                    transform.translation.y = to.1;
                }
                return;
            }

            let to_animate = if time
                .time_since_startup()
                .lt(&(CAMERA_TO_GOAL + game_state.scene_start))
            {
                (Vec2::new(to.0 - from.0, to.1 - from.1)
                    / (CAMERA_TO_GOAL - CAMERA_ON_PLAYER).as_secs_f32())
                    * time.delta().as_secs_f32()
            } else {
                (Vec2::new(from.0 - to.0, from.1 - to.1)
                    / (CAMERA_BACK_TO_PLAYER - CAMERA_ON_GOAL).as_secs_f32())
                    * time.delta().as_secs_f32()
            };
            if let Ok(mut transform) = camera.single_mut() {
                transform.translation.x += to_animate.x;
                transform.translation.y += to_animate.y;
            }
        }
    }
}

fn run_transition_scene(
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
    mut current_map: ResMut<Map>,
    mut player: Query<&mut Transform, (With<Player>, Without<PlayerCamera>)>,
    mut camera: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    if let Some(scene) = &game_state.scene {
        if let CutScene::MapTransition { to } = scene {
            let camera_scale_offset: Vec3 = Vec3::new(-0.98, -0.98, 0.);
            let player_scale_offset: Vec3 = Vec3::new(-0.95, -0.95, 0.);
            const ZOOM: Duration = Duration::from_secs(2);

            if time
                .time_since_startup()
                .lt(&(game_state.scene_start + ZOOM))
            {
                if let Ok(mut transform) = camera.single_mut() {
                    transform.scale +=
                        (camera_scale_offset / ZOOM.as_secs_f32()) * time.delta().as_secs_f32();
                }
                if let Ok(mut transform) = player.single_mut() {
                    transform.scale +=
                        (player_scale_offset / ZOOM.as_secs_f32()) * time.delta().as_secs_f32();
                }
                return;
            }
            *current_map = to.clone();
            game_state.scene = None;
            game_state.frozen = false;
            return;
        }
    }
}

fn trigger_scene(
    time: Res<Time>,
    mut trigger_scene: EventReader<TriggerScene>,
    mut game_state: ResMut<GameState>,
) {
    for event in trigger_scene.iter() {
        game_state.scene = Some(event.scene.clone());
        game_state.frozen = true;
        game_state.scene_start = time.time_since_startup();
    }
}
