use crate::loading::TextureAssets;
use crate::player::calc_camera_position;
use crate::scenes::{CutScene, TriggerScene};
use crate::{GameStage, TiledMap};
use bevy::prelude::*;
use std::collections::HashMap;
use tiled::LayerData::Finite;
use tiled::PropertyValue::BoolValue;

pub const TILE_SIZE: f32 = 64.;
pub const ACTIVE_ELEMENT_Z: f32 = 2.;

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

#[derive(Clone)]
pub enum Map {
    Ground,
    Dirt,
    Stone,
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
        }
    }

    pub fn goal_position(&self) -> (f32, f32) {
        match self {
            Map::Ground => self.position_from_slot(Slot { column: 9, row: 10 }),
            Map::Dirt => self.position_from_slot(Slot { column: 12, row: 8 }),
            Map::Stone => self.position_from_slot(Slot { column: 14, row: 5 }),
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
        match self {
            _ => {
                let from = calc_camera_position(
                    self.start_position().0,
                    self.start_position().1,
                    window,
                    &self.dimensions(),
                );
                let to = calc_camera_position(
                    self.goal_position().0,
                    self.goal_position().1,
                    window,
                    &self.dimensions(),
                );

                Some(CutScene::Intro { from, to })
            }
        }
    }

    pub fn goal_scene(&self) -> Option<CutScene> {
        match self {
            Map::Ground => Some(CutScene::MapTransition { to: Map::Dirt }),
            Map::Dirt => Some(CutScene::MapTransition { to: Map::Stone }),
            Map::Stone => None,
        }
    }

    pub fn active_elements(&self) -> Vec<ActiveElement> {
        match self {
            Map::Ground => vec![ActiveElement::Button {
                position: self.tiled_slot_to_bevy_slot(Slot { row: 7, column: 5 }),
                connected_wall: self.tiled_slot_to_bevy_slot(Slot { row: 10, column: 8 }),
            }],
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
    let window = windows.get_primary().expect("No primary window");
    if let Some(scene) = current_map.intro_scene(window) {
        trigger_scene.send(TriggerScene { scene });
    }
}

fn draw_active_elements(
    mut commands: Commands,
    current_map: Res<Map>,
    asset_server: Res<AssetServer>,
    elements: Query<Entity, With<ActiveElement>>,
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
            commands
                .spawn_bundle(SpriteBundle {
                    material: materials.add(textures.texture_button.clone().into()),
                    transform: Transform::from_translation(Vec3::new(
                        position.column as f32 * TILE_SIZE,
                        position.row as f32 * TILE_SIZE,
                        ACTIVE_ELEMENT_Z,
                    )),
                    ..Default::default()
                })
                .insert(element.clone());
            commands
                .spawn_bundle(SpriteBundle {
                    material: materials
                        .add(asset_server.get_handle("textures/stonewall.png").into()),
                    transform: Transform::from_translation(Vec3::new(
                        connected_wall.column as f32 * TILE_SIZE,
                        connected_wall.row as f32 * TILE_SIZE,
                        ACTIVE_ELEMENT_Z,
                    )),
                    ..Default::default()
                })
                .insert(element.clone());
        }
    }
}
