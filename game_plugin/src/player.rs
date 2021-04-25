use crate::actions::Actions;
use crate::loading::TextureAssets;
use crate::map::{Collide, Dimensions, Map, MapSystemLabels, TILE_SIZE};
use crate::{GameStage, GameState};
use bevy::prelude::*;

pub struct PlayerPlugin;

pub struct Player;
pub struct PlayerCamera;

#[derive(SystemLabel, Clone, Hash, Debug, Eq, PartialEq)]
pub enum PlayerSystemLabels {
    MovePlayer,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_enter(GameStage::Playing)
                .after(MapSystemLabels::DrawMap)
                .with_system(spawn_player.system())
                .with_system(spawn_camera.system()),
        )
        .add_system_set(
            SystemSet::on_update(GameStage::Playing)
                .with_system(move_player.system().label(PlayerSystemLabels::MovePlayer))
                .with_system(move_camera.system().after(PlayerSystemLabels::MovePlayer)),
        )
        .add_system_set(SystemSet::on_exit(GameStage::Playing).with_system(remove_player.system()));
    }
}

fn spawn_camera(mut commands: Commands, current_map: Res<Map>, windows: Res<Windows>) {
    let (x, y) = current_map.position_from_slot(current_map.start_slot());
    let window = windows.get_primary().expect("No primary window");
    let (x, y) = calc_camera_position(x, y, window, &current_map.dimensions());

    commands
        .spawn_bundle(OrthographicCameraBundle {
            transform: Transform::from_xyz(x, y, 1000. - 0.1),
            ..OrthographicCameraBundle::new_2d()
        })
        .insert(PlayerCamera);
}

fn spawn_player(
    mut commands: Commands,
    current_map: Res<Map>,
    textures: Res<TextureAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let spawn_position: (f32, f32) = current_map.position_from_slot(current_map.start_slot());
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(textures.texture_player.clone().into()),
            transform: Transform::from_translation(Vec3::new(
                spawn_position.0,
                spawn_position.1,
                1.,
            )),
            ..Default::default()
        })
        .insert(Player);
}

fn move_player(
    time: Res<Time>,
    game_state: Res<GameState>,
    actions: Res<Actions>,
    map: Res<Map>,
    mut player_query: Query<&mut Transform, (With<Player>, Without<PlayerCamera>)>,
    collider_query: Query<&Collide>,
) {
    if actions.player_movement.is_none() || game_state.frozen {
        return;
    }
    let speed = 150.;
    let movement = Vec3::new(
        actions.player_movement.unwrap().x * speed * time.delta_seconds(),
        actions.player_movement.unwrap().y * speed * time.delta_seconds(),
        0.,
    );
    for mut player_transform in player_query.iter_mut() {
        let x =
            ((player_transform.translation.x + movement.x + TILE_SIZE / 2.) / TILE_SIZE) as usize;
        let y =
            ((player_transform.translation.y + movement.y + TILE_SIZE / 2.) / TILE_SIZE) as usize;
        if x >= map.dimensions().columns || y >= map.dimensions().rows {
            return;
        }
        for collide in collider_query.iter() {
            if collide.x == x && collide.y == y {
                return;
            }
        }
        player_transform.translation += movement;
    }
}

fn move_camera(
    map: Res<Map>,
    game_state: Res<GameState>,
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
