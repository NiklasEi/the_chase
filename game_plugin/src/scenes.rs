use crate::map::Map;
use crate::player::{calc_camera_position, PlayerCamera};
use crate::{GameStage, GameState};
use bevy::ecs::schedule::ShouldRun;
use bevy::ecs::world::WorldCell;
use bevy::prelude::*;
use std::time::Duration;

pub struct GroundIntro;

pub struct ScenesPlugin;

pub struct TriggerScene {
    pub scene: CutScene,
}

#[derive(Clone, PartialEq)]
pub enum CutScene {
    GroundIntro,
}

impl Plugin for ScenesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<TriggerScene>().add_system_set(
            SystemSet::on_update(GameStage::Playing)
                .with_system(run_ground_intro.system())
                .with_system(trigger_scene.system()),
        );
    }
}

fn run_ground_intro(
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
    windows: Res<Windows>,
    mut camera: Query<&mut Transform, With<PlayerCamera>>,
) {
    if let Some(scene) = &game_state.scene {
        if scene != &CutScene::GroundIntro {
            return;
        }
    } else {
        return;
    }
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

    let window = windows.get_primary().expect("No primary window");
    let starting_point = Map::Ground.position_from_slot(Map::Ground.start_slot());
    let (x_start, y_start) = calc_camera_position(
        starting_point.0,
        starting_point.1,
        window,
        &Map::Ground.dimensions(),
    );

    if time
        .time_since_startup()
        .gt(&(game_state.scene_start + CAMERA_BACK_TO_PLAYER))
    {
        if let Ok(mut transform) = camera.single_mut() {
            transform.translation.x = x_start;
            transform.translation.y = y_start;
        }
        game_state.scene = None;
        game_state.frozen = false;
        return;
    }

    let goal_point = Map::Ground.position_from_slot(Map::Ground.goal_slot());
    let (x_goal, y_goal) = calc_camera_position(
        goal_point.0,
        goal_point.1,
        window,
        &Map::Ground.dimensions(),
    );

    if time
        .time_since_startup()
        .gt(&(game_state.scene_start + CAMERA_TO_GOAL))
        && time
            .time_since_startup()
            .lt(&(game_state.scene_start + CAMERA_ON_GOAL))
    {
        if let Ok(mut transform) = camera.single_mut() {
            transform.translation.x = x_goal;
            transform.translation.y = y_goal;
        }
        return;
    }

    let to_animate = if time
        .time_since_startup()
        .lt(&(CAMERA_TO_GOAL + game_state.scene_start))
    {
        (Vec2::new(x_goal - x_start, y_goal - y_start)
            / (CAMERA_TO_GOAL - CAMERA_ON_PLAYER).as_secs_f32())
            * time.delta().as_secs_f32()
    } else {
        (Vec2::new(x_start - x_goal, y_start - y_goal)
            / (CAMERA_BACK_TO_PLAYER - CAMERA_ON_GOAL).as_secs_f32())
            * time.delta().as_secs_f32()
    };
    if let Ok(mut transform) = camera.single_mut() {
        transform.translation.x += to_animate.x;
        transform.translation.y += to_animate.y;
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
