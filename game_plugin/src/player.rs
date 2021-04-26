use crate::actions::Actions;
use crate::audio::BackgroundAudio;
use crate::loading::{AudioAssets, TextureAssets};
use crate::map::{Collide, Dimensions, Map, MapSystemLabels, TILE_SIZE};
use crate::scenes::TriggerScene;
use crate::{GameData, GameState};
use bevy::prelude::*;
use std::f32::consts::PI;
use std::ops::Deref;

pub struct PlayerPlugin;

pub struct Player;
pub struct PlayerCamera;

pub const PLAYER_Z: f32 = 5.;

#[derive(SystemLabel, Clone, Hash, Debug, Eq, PartialEq)]
pub enum PlayerSystemLabels {
    MovePlayer,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_enter(GameState::Playing)
                .after(MapSystemLabels::DrawMap)
                .with_system(spawn_player.system()),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(reset_player_position.system())
                .with_system(move_player.system().label(PlayerSystemLabels::MovePlayer))
                .with_system(move_camera.system().after(PlayerSystemLabels::MovePlayer)),
        )
        .add_system_set(SystemSet::on_exit(GameState::Playing).with_system(remove_player.system()));
    }
}

fn spawn_player(
    mut commands: Commands,
    current_map: Res<Map>,
    textures: Res<TextureAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let spawn_position: (f32, f32) = current_map.start_position();
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(textures.texture_player.clone().into()),
            transform: Transform::from_translation(Vec3::new(
                spawn_position.0,
                spawn_position.1,
                PLAYER_Z,
            )),
            ..Default::default()
        })
        .insert(Player);
}

fn move_player(
    time: Res<Time>,
    game_state: Res<GameData>,
    actions: Res<Actions>,
    map: Res<Map>,
    mut trigger_scene: EventWriter<TriggerScene>,
    mut player_query: Query<&mut Transform, (With<Player>, Without<PlayerCamera>)>,
    collider_query: Query<&Collide>,
) {
    if actions.player_movement.is_none() || game_state.frozen {
        return;
    }
    let speed = 250.;
    let movement = Vec3::new(
        actions.player_movement.unwrap().x * speed * time.delta_seconds(),
        actions.player_movement.unwrap().y * speed * time.delta_seconds(),
        0.,
    );
    let player_bounds = movement.normalize() * 8.;
    for mut player_transform in player_query.iter_mut() {
        player_transform.rotation = Quat::from_rotation_z(
            -1. * actions
                .player_movement
                .unwrap()
                .angle_between(Vec2::new(0., 1.))
                + PI,
        );
        let x = ((player_transform.translation.x + movement.x + player_bounds.x + TILE_SIZE / 2.)
            / TILE_SIZE) as usize;
        let y = ((player_transform.translation.y + movement.y + player_bounds.y + TILE_SIZE / 2.)
            / TILE_SIZE) as usize;
        if x >= map.dimensions().columns || y >= map.dimensions().rows {
            return;
        }
        for collide in collider_query.iter() {
            if collide.x == x && collide.y == y {
                return;
            }
        }
        player_transform.translation += movement;
        if player_transform.translation.distance(Vec3::new(
            map.goal_position().0,
            map.goal_position().1,
            PLAYER_Z,
        )) < 25.
        {
            if let Some(scene) = map.goal_scene((
                player_transform.translation.x,
                player_transform.translation.y,
            )) {
                trigger_scene.send(TriggerScene { scene });
            }
        }
    }
}

fn reset_player_position(
    current_map: Res<Map>,
    windows: Res<Windows>,
    audio_assets: Res<AudioAssets>,
    mut background_audio: EventWriter<BackgroundAudio>,
    mut player_query: Query<&mut Transform, (With<Player>, Without<PlayerCamera>)>,
    mut camera_query: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    if current_map.is_changed() {
        let window = windows.get_primary().expect("No primary window");
        let spawn_position: (f32, f32) = current_map.start_position();
        if let Ok(mut player_transform) = player_query.single_mut() {
            player_transform.translation.x = spawn_position.0;
            player_transform.translation.y = spawn_position.1;
            player_transform.scale = Vec3::new(1., 1., 1.);
        }
        if let Ok(mut camera_transform) = camera_query.single_mut() {
            let (x, y) = calc_camera_position(
                spawn_position.0,
                spawn_position.1,
                window,
                &current_map.dimensions(),
            );
            camera_transform.translation.x = x;
            camera_transform.translation.y = y;
            camera_transform.scale = Vec3::new(1., 1., 1.);
        }
        match current_map.deref() {
            Map::Dirt => background_audio.send(BackgroundAudio {
                handles: vec![audio_assets.dirt_background.clone()],
            }),
            Map::Stone => background_audio.send(BackgroundAudio {
                handles: vec![audio_assets.stone_background.clone()],
            }),
            Map::Lava => background_audio.send(BackgroundAudio {
                handles: vec![
                    audio_assets.lava_background.clone(),
                    audio_assets.lava_background_effects.clone(),
                ],
            }),
            Map::Ground => background_audio.send(BackgroundAudio {
                handles: vec![
                    audio_assets.ground_background.clone(),
                    audio_assets.ground_background_effects.clone(),
                ],
            }),
        }
    }
}

fn move_camera(
    map: Res<Map>,
    game_state: Res<GameData>,
    actions: Res<Actions>,
    windows: Res<Windows>,
    player_query: Query<&Transform, (With<Player>, Without<PlayerCamera>)>,
    mut camera_query: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    if actions.player_movement.is_none() || game_state.frozen {
        return;
    }
    if let Ok(player_transform) = player_query.single() {
        let window = windows.get_primary().expect("No primary window");
        let (x, y) = calc_camera_position(
            player_transform.translation.x,
            player_transform.translation.y,
            window,
            &map.dimensions(),
        );

        if let Ok(mut camera_transform) = camera_query.single_mut() {
            camera_transform.translation.x = x;
            camera_transform.translation.y = y;
        }
    }
}

pub fn calc_camera_position(
    mut x: f32,
    mut y: f32,
    window: &Window,
    map_dimensions: &Dimensions,
) -> (f32, f32) {
    let x_min = window.width() / 2. - TILE_SIZE / 2.;
    let x_max = map_dimensions.columns as f32 * TILE_SIZE - window.width() / 2. - TILE_SIZE / 2.;
    let y_min = window.height() / 2. - TILE_SIZE / 2.;
    let y_max = map_dimensions.rows as f32 * TILE_SIZE - window.height() / 2. - TILE_SIZE / 2.;

    if x_min < x_max {
        x = x.clamp(x_min, x_max);
    } else {
        x = ((map_dimensions.columns - 1) as f32 * TILE_SIZE) / 2.;
    }
    if y_min < y_max {
        y = y.clamp(y_min, y_max);
    } else {
        y = ((map_dimensions.rows - 1) as f32 * TILE_SIZE) / 2.;
    }
    (x, y)
}

fn remove_player(mut commands: Commands, player_query: Query<Entity, With<Player>>) {
    for player in player_query.iter() {
        commands.entity(player).despawn();
    }
}
