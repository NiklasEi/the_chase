use crate::loading::TextureAssets;
use crate::player::{calc_camera_position, Player};
use crate::scenes::{CutScene, TriggerScene};
use crate::{GameStage, GameState, TiledMap};
use bevy::prelude::*;
use std::collections::HashMap;
use tiled::LayerData::Finite;
use tiled::PropertyValue::BoolValue;

pub const TILE_SIZE: f32 = 64.;
pub const ACTIVE_ELEMENT_Z: f32 = 2.;
pub const ACORN_Z: f32 = 1.;

#[derive(SystemLabel, Clone, Hash, Debug, Eq, PartialEq)]
pub enum MapSystemLabels {
    DrawMap,
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(Map::Ground)
            .add_system_set(
                SystemSet::on_update(GameStage::Playing)
                    .label(MapSystemLabels::DrawMap)
                    .with_system(load_map.system().chain(draw_map.system())),
            )
            .add_system_set(
                SystemSet::on_update(GameStage::Playing)
                    .with_system(draw_active_elements.system())
                    .with_system(check_active_elements.system())
                    .after(MapSystemLabels::DrawMap),
            );
    }
}

pub struct MapData {
    layers: Vec<Vec<Vec<Tile>>>,
    colliding_layers: Vec<bool>,
    height: usize,
    width: usize,
}

#[derive(Debug, Clone)]
pub struct Slot {
    pub column: usize,
    pub row: usize,
}

pub struct Dimensions {
    pub columns: usize,
    pub rows: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Tile {
    pub asset_path: Option<String>,
}

pub struct MapTile {
    pub column: usize,
    pub row: usize,
    pub tile: Tile,
}

pub struct Collide {
    pub x: usize,
    pub y: usize,
}

pub struct Acorn;

#[derive(Clone)]
pub enum Map {
    Ground,
    Dirt,
    Stone,
    Lava,
}

impl Map {
    fn file(&self) -> String {
        format!("map/{}.tmx", self.name())
    }

    fn name(&self) -> &str {
        match self {
            Map::Ground => "ground",
            Map::Dirt => "dirt",
            Map::Stone => "stone",
            Map::Lava => "lava",
        }
    }

    pub fn start_position(&self) -> (f32, f32) {
        match self {
            Map::Ground => self.position_from_slot(Slot { column: 1, row: 0 }),
            Map::Dirt => self.position_from_slot(Slot {
                column: 18,
                row: 16,
            }),
            Map::Stone => self.position_from_slot(Slot { column: 7, row: 12 }),
            Map::Lava => self.position_from_slot(Slot { column: 6, row: 23 }),
        }
    }

    pub fn goal_position(&self) -> (f32, f32) {
        match self {
            Map::Ground => self.position_from_slot(Slot { column: 9, row: 10 }),
            Map::Dirt => self.position_from_slot(Slot { column: 12, row: 8 }),
            Map::Stone => self.position_from_slot(Slot { column: 14, row: 5 }),
            Map::Lava => self.position_from_slot(Slot {
                column: 22,
                row: 22,
            }),
        }
    }

    pub fn acorn_position(&self) -> (f32, f32) {
        match self {
            Map::Ground => self.position_from_slot(Slot { column: 8, row: 10 }),
            Map::Dirt => self.position_from_slot(Slot { column: 10, row: 8 }),
            Map::Stone => self.position_from_slot(Slot { column: 14, row: 7 }),
            Map::Lava => self.position_from_slot(Slot {
                column: 22,
                row: 20,
            }),
        }
    }

    pub fn dimensions(&self) -> Dimensions {
        match self {
            Map::Ground => Dimensions {
                columns: 20,
                rows: 20,
            },
            Map::Dirt => Dimensions {
                columns: 20,
                rows: 20,
            },
            Map::Stone => Dimensions {
                columns: 20,
                rows: 20,
            },
            Map::Lava => Dimensions {
                columns: 30,
                rows: 30,
            },
        }
    }

    pub fn tiled_slot_to_bevy_slot(&self, slot: Slot) -> Slot {
        Slot {
            column: slot.column,
            row: self.dimensions().rows - slot.row - 1,
        }
    }

    pub fn position_from_slot(&self, slot: Slot) -> (f32, f32) {
        let dimensions = self.dimensions();
        (
            slot.column as f32 * TILE_SIZE,
            (dimensions.rows - slot.row - 1) as f32 * TILE_SIZE,
        )
    }

    pub fn intro_scene(&self, window: &Window) -> Option<CutScene> {
        let camera_from = calc_camera_position(
            self.start_position().0,
            self.start_position().1,
            window,
            &self.dimensions(),
        );
        let camera_to = calc_camera_position(
            self.goal_position().0,
            self.goal_position().1,
            window,
            &self.dimensions(),
        );
        match self {
            Map::Lava => Some(CutScene::Intro {
                camera_from,
                camera_to,
                acorn_falls: false,
            }),
            _ => Some(CutScene::Intro {
                camera_from,
                camera_to,
                acorn_falls: true,
            }),
        }
    }

    pub fn goal_scene(&self, from: (f32, f32)) -> Option<CutScene> {
        match self {
            Map::Ground => Some(CutScene::MapTransition {
                to: Map::Dirt,
                camera_to: self.goal_position(),
                camera_from: from,
            }),
            Map::Dirt => Some(CutScene::MapTransition {
                to: Map::Stone,
                camera_to: self.goal_position(),
                camera_from: from,
            }),
            Map::Stone => Some(CutScene::MapTransition {
                to: Map::Lava,
                camera_to: self.goal_position(),
                camera_from: from,
            }),
            Map::Lava => Some(CutScene::Won),
        }
    }

    pub fn active_elements(&self) -> Vec<ActiveElement> {
        match self {
            Map::Ground => vec![
                ActiveElement::Button {
                    position: Slot { column: 5, row: 7 },
                    connected_wall: Slot { column: 2, row: 17 },
                },
                ActiveElement::Button {
                    position: Slot { column: 13, row: 9 },
                    connected_wall: Slot { column: 7, row: 10 },
                },
            ],
            _ => vec![],
        }
    }
}

#[derive(Clone)]
pub enum ActiveElement {
    Button {
        position: Slot,
        connected_wall: Slot,
    },
}
pub struct Trigger;

fn load_map(current_map: Res<Map>, maps: Res<Assets<TiledMap>>) -> Option<MapData> {
    if !current_map.is_added() && !current_map.is_changed() {
        return None;
    }
    if let Some(map) = maps.get(&current_map.file()[..]) {
        let map = &map.map;
        let mut path_map: HashMap<u32, String> = HashMap::default();
        for set in map.tilesets.iter() {
            for tile in set.tiles.iter() {
                path_map.insert(
                    set.first_gid + tile.id,
                    tile.images.first().unwrap().source.clone(),
                );
            }
        }

        let mut layers = vec![];
        for layer in map.layers.iter() {
            let mut current_layer = vec![];
            if let Finite(tiles) = &layer.tiles {
                for row in tiles {
                    let mut current_row = vec![];
                    for tile in row {
                        current_row.push(tile.gid);
                    }
                    current_layer.push(current_row);
                }
            }
            layers.push(current_layer);
        }
        let colliding: Vec<usize> = map
            .layers
            .iter()
            .enumerate()
            .filter(|(_index, layer)| {
                if let Some(BoolValue(collide)) = layer.properties.get("collide") {
                    return collide.clone();
                }
                false
            })
            .map(|(index, _layer)| index)
            .collect();
        let mut colliding_layers: Vec<bool> = vec![];
        let mut tile_layers: Vec<Vec<Vec<Tile>>> = vec![];
        for (floor_index, layer_data) in layers.iter().enumerate() {
            let mut floor = vec![];
            for (_row_index, row_data) in layer_data.iter().enumerate() {
                let mut row: Vec<Tile> = vec![];
                for (_column_index, gid) in row_data.iter().enumerate() {
                    if let Some(path) = path_map.get(gid) {
                        row.push(Tile {
                            asset_path: Some(path.clone()),
                        })
                    } else {
                        row.push(Tile { asset_path: None })
                    }
                }
                floor.push(row);
            }
            // otherwise the map is upside down O.o
            floor.reverse();
            colliding_layers.push(colliding.contains(&floor_index));
            tile_layers.push(floor);
        }
        return Some(MapData {
            layers: tile_layers,
            height: map.height as usize,
            width: map.width as usize,
            colliding_layers,
        });
    }
    None
}

fn draw_map(
    map_data: In<Option<MapData>>,
    mut commands: Commands,
    current_map: Res<Map>,
    asset_server: Res<AssetServer>,
    windows: Res<Windows>,
    texture_assets: Res<TextureAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut trigger_scene: EventWriter<TriggerScene>,
    tiles: Query<Entity, With<MapTile>>,
) {
    if map_data.0.is_none() {
        return;
    }
    for entity in tiles.iter() {
        commands.entity(entity).despawn();
    }
    let map_data: MapData = map_data.0.unwrap();
    for (layer_index, layer) in map_data.layers.iter().enumerate() {
        let collide = map_data
            .colliding_layers
            .get(layer_index)
            .unwrap_or(&false)
            .clone();
        for row in 0..map_data.height {
            for column in 0..map_data.width {
                let tile = &layer[row][column];
                if let Some(path) = &tile.asset_path {
                    let sprite = SpriteBundle {
                        material: materials.add(asset_server.get_handle(&(path)[3..]).into()),
                        transform: Transform::from_translation(Vec3::new(
                            column as f32 * TILE_SIZE,
                            row as f32 * TILE_SIZE,
                            0.,
                        )),
                        ..Default::default()
                    };
                    let tile = MapTile {
                        column,
                        row,
                        tile: tile.clone(),
                    };
                    if collide {
                        commands
                            .spawn_bundle(sprite)
                            .insert(tile)
                            .insert(Collide { x: column, y: row });
                    } else {
                        commands.spawn_bundle(sprite).insert(tile);
                    }
                }
            }
        }
    }
    let acorn_position = current_map.acorn_position();
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_assets.texture_acorn.clone().into()),
            transform: Transform::from_translation(Vec3::new(
                acorn_position.0,
                acorn_position.1,
                ACORN_Z,
            )),
            ..Default::default()
        })
        .insert(Acorn);
    let window = windows.get_primary().expect("No primary window");
    if let Some(scene) = current_map.intro_scene(window) {
        trigger_scene.send(TriggerScene { scene });
    }
}

pub struct ButtonWall {
    button_slot: Slot,
    wall_slot: Slot,
    button: Entity,
    wall: Entity,
}

fn draw_active_elements(
    mut commands: Commands,
    current_map: Res<Map>,
    elements: Query<Entity, With<ButtonWall>>,
    textures: Res<TextureAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !current_map.is_added() && !current_map.is_changed() {
        return;
    }
    for entity in elements.iter() {
        commands.entity(entity).despawn();
    }
    let active_elements = current_map.active_elements();
    for element in active_elements {
        if let ActiveElement::Button {
            position,
            connected_wall,
        } = element.clone()
        {
            let button_slot = current_map.tiled_slot_to_bevy_slot(position.clone());
            let connected_wall_slot = current_map.tiled_slot_to_bevy_slot(connected_wall.clone());
            let button = commands
                .spawn_bundle(SpriteBundle {
                    material: materials.add(textures.texture_button_up.clone().into()),
                    transform: Transform::from_translation(Vec3::new(
                        button_slot.column as f32 * TILE_SIZE,
                        button_slot.row as f32 * TILE_SIZE,
                        ACTIVE_ELEMENT_Z,
                    )),
                    ..Default::default()
                })
                .id();
            commands.entity(button).insert(Trigger);
            let wall = commands
                .spawn_bundle(SpriteBundle {
                    material: materials.add(textures.texture_wall_up.clone().into()),
                    transform: Transform::from_translation(Vec3::new(
                        connected_wall_slot.column as f32 * TILE_SIZE,
                        connected_wall_slot.row as f32 * TILE_SIZE,
                        ACTIVE_ELEMENT_Z,
                    )),
                    ..Default::default()
                })
                .id();
            commands.entity(wall).insert(Collide {
                x: connected_wall_slot.column,
                y: connected_wall_slot.row,
            });

            commands.entity(wall).insert(ButtonWall {
                button: button.clone(),
                wall: wall.clone(),
                button_slot: position.clone(),
                wall_slot: connected_wall.clone(),
            });
            commands.entity(button).insert(ButtonWall {
                button: button.clone(),
                wall: wall.clone(),
                button_slot: position.clone(),
                wall_slot: connected_wall.clone(),
            });
        }
    }
}

fn check_active_elements(
    mut commands: Commands,
    current_map: Res<Map>,
    game_state: Res<GameState>,
    windows: Res<Windows>,
    mut elements: Query<(Entity, &Transform, &ButtonWall), With<Trigger>>,
    player_query: Query<&Transform, With<Player>>,
    mut trigger_scene: EventWriter<TriggerScene>,
) {
    if game_state.frozen {
        return;
    }
    if let Ok(player_transform) = player_query.single() {
        for (entity, transform, element) in elements.iter_mut() {
            if Vec2::new(
                player_transform.translation.x,
                player_transform.translation.y,
            )
            .distance(Vec2::new(transform.translation.x, transform.translation.y))
                < 25.
            {
                commands.entity(entity).remove::<Trigger>();
                let ButtonWall {
                    button_slot: _button_slot,
                    wall_slot,
                    button,
                    wall,
                } = element;

                let wall_position = current_map.position_from_slot(wall_slot.clone());
                let window = windows.get_primary().expect("No primary window");
                trigger_scene.send(TriggerScene {
                    scene: CutScene::ActivateButton {
                        button: button.clone(),
                        wall: wall.clone(),
                        camera_from: calc_camera_position(
                            player_transform.translation.x,
                            player_transform.translation.y,
                            window,
                            &current_map.dimensions(),
                        ),
                        camera_to: calc_camera_position(
                            wall_position.0,
                            wall_position.1,
                            window,
                            &current_map.dimensions(),
                        ),
                    },
                });
            }
        }
    }
}
