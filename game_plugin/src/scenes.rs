use crate::audio::{AudioEffect, PauseBackground, ResumeBackground};
use crate::loading::AudioAssets;
use crate::map::{ActiveElement, Collide, Map};
use crate::player::{Player, PlayerCamera};
use crate::{GameStage, GameState};
use bevy::prelude::*;
use std::time::Duration;

pub struct ScenesPlugin;

pub struct TriggerScene {
    pub scene: CutScene,
}

#[derive(Clone)]
pub enum CutScene {
    Intro {
        from: (f32, f32),
        to: (f32, f32),
    },
    ActivateButton {
        button: (f32, f32),
        wall: (f32, f32),
    },
    MapTransition {
        to: Map,
    },
}

impl Plugin for ScenesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<TriggerScene>().add_system_set(
            SystemSet::on_update(GameStage::Playing)
                .with_system(run_intro.system())
                .with_system(run_transition_scene.system())
                .with_system(run_activate_button_scene.system())
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
    mut audio_effect: EventWriter<AudioEffect>,
    audio_assets: Res<AudioAssets>,
    mut pause_background: EventWriter<PauseBackground>,
    mut player: Query<&mut Transform, (With<Player>, Without<PlayerCamera>)>,
    mut camera: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    if let Some(scene) = game_state.scene.clone() {
        if let CutScene::MapTransition { to } = scene {
            if game_state.scene_step == 0 {
                game_state.scene_step += 1;
                pause_background.send(PauseBackground);
                audio_effect.send(AudioEffect {
                    handle: audio_assets.fall.clone(),
                })
            }
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

fn run_activate_button_scene(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
    mut audio_effect: EventWriter<AudioEffect>,
    audio_assets: Res<AudioAssets>,
    mut pause_background: EventWriter<PauseBackground>,
    mut resume_background: EventWriter<ResumeBackground>,
    elements: Query<(Entity, &Transform), (With<ActiveElement>, Without<PlayerCamera>)>,
    mut camera: Query<&mut Transform, (With<PlayerCamera>, Without<ActiveElement>)>,
) {
    if let Some(scene) = game_state.scene.clone() {
        if let CutScene::ActivateButton { button, wall } = scene {
            if game_state.scene_step == 0 {
                game_state.scene_step += 1;
                pause_background.send(PauseBackground);
                audio_effect.send(AudioEffect {
                    handle: audio_assets.button_click.clone(),
                })
            }
            const CAMERA_ON_PLAYER: Duration = Duration::from_millis(300);
            const CAMERA_TO_WALL: Duration = Duration::from_millis(1000);
            const CAMERA_ON_WALL: Duration = Duration::from_millis(1300);
            const CAMERA_BACK_TO_PLAYER: Duration = Duration::from_millis(2000);

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
                resume_background.send(ResumeBackground);
                if let Ok(mut transform) = camera.single_mut() {
                    transform.translation.x = button.0;
                    transform.translation.y = button.1;
                }
                game_state.scene = None;
                game_state.frozen = false;
                return;
            }

            if time
                .time_since_startup()
                .gt(&(game_state.scene_start + CAMERA_TO_WALL))
                && time
                    .time_since_startup()
                    .lt(&(game_state.scene_start + CAMERA_ON_WALL))
            {
                if game_state.scene_step == 1 {
                    game_state.scene_step += 1;
                    audio_effect.send(AudioEffect {
                        handle: audio_assets.wall_moving.clone(),
                    })
                }
                if let Ok(mut transform) = camera.single_mut() {
                    transform.translation.x = wall.0;
                    transform.translation.y = wall.1;
                }
                return;
            }

            if game_state.scene_step == 2 {
                game_state.scene_step += 1;
                for (entity, transform) in elements.iter() {
                    if transform.translation.x == wall.0 && transform.translation.y == wall.1 {
                        // ToDo: change texture; make wall disappear
                        commands.entity(entity).remove::<Collide>();
                    }
                }
            }

            let to_animate = if time
                .time_since_startup()
                .lt(&(CAMERA_TO_WALL + game_state.scene_start))
            {
                (Vec2::new(wall.0 - button.0, wall.1 - button.1)
                    / (CAMERA_TO_WALL - CAMERA_ON_PLAYER).as_secs_f32())
                    * time.delta().as_secs_f32()
            } else {
                (Vec2::new(button.0 - wall.0, button.1 - wall.1)
                    / (CAMERA_BACK_TO_PLAYER - CAMERA_ON_WALL).as_secs_f32())
                    * time.delta().as_secs_f32()
            };
            if let Ok(mut transform) = camera.single_mut() {
                transform.translation.x += to_animate.x;
                transform.translation.y += to_animate.y;
            }
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
        game_state.scene_step = 0;
    }
}
