use crate::{GameState, TiledMap};
use bevy::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tiled::LayerData::Finite;
use tiled::PropertyValue::BoolValue;

pub const TILE_SIZE: f32 = 64.;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(Map::Earth).add_system_set(
            SystemSet::on_enter(GameState::Playing)
                .with_system(load_map.system().chain(draw_map.system())),
        );
    }
}

pub struct MapData {
    layers: Vec<Vec<Vec<Tile>>>,
    colliding_layers: Vec<bool>,
    height: usize,
    width: usize,
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

pub enum Map {
    Ground,
    Earth,
}

impl Map {
    fn file(&self) -> String {
        format!("map/{}.tmx", self.name())
    }

    fn name(&self) -> &str {
        match self {
            Map::Ground => "ground",
            Map::Earth => "earth",
        }
    }
}

fn load_map(current_map: Res<Map>, maps: Res<Assets<TiledMap>>) -> Option<MapData> {
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
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let map_data: MapData = map_data.0.expect("There is no map O.o");
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
}
