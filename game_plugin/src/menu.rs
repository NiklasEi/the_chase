use crate::audio::BackgroundAudio;
use crate::loading::{AudioAssets, TextureAssets};
use crate::map::TILE_SIZE;
use crate::player::PlayerCamera;
use crate::GameState;
use bevy::prelude::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<ButtonMaterials>()
            .add_system_set(SystemSet::on_enter(GameState::Menu).with_system(setup_menu.system()))
            .add_system_set(
                SystemSet::on_update(GameState::Menu).with_system(click_play_button.system()),
            );
    }
}

pub struct ButtonMaterials {
    pub normal: Handle<ColorMaterial>,
    pub hovered: Handle<ColorMaterial>,
}

impl FromWorld for ButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
        }
    }
}

struct PlayButton;
struct Menu;

fn setup_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    audio_assets: Res<AudioAssets>,
    texture_assets: Res<TextureAssets>,
    mut background_audio: EventWriter<BackgroundAudio>,
) {
    background_audio.send(BackgroundAudio {
        handles: vec![
            audio_assets.ground_background.clone(),
            audio_assets.ground_background_effects.clone(),
        ],
    });
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(PlayerCamera);
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_assets.texture_menu.clone().into()),
            transform: Transform::from_translation(Vec3::new(TILE_SIZE / 2., TILE_SIZE / 2., 1.)),
            ..Default::default()
        })
        .insert(Menu);
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_assets.texture_player.clone().into()),
            transform: Transform::from_translation(Vec3::new(2. * TILE_SIZE, 2. * TILE_SIZE, 2.)),
            ..Default::default()
        })
        .insert(Menu);
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_assets.texture_acorn.clone().into()),
            transform: Transform::from_translation(Vec3::new(-2. * TILE_SIZE, -2. * TILE_SIZE, 2.)),
            ..Default::default()
        })
        .insert(Menu);
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(120.0), Val::Px(50.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: button_materials.normal.clone(),
            ..Default::default()
        })
        .insert(PlayButton)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "Play".to_string(),
                        style: TextStyle {
                            font: asset_server.get_handle("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    }],
                    alignment: Default::default(),
                },
                ..Default::default()
            });
        });
}

type ButtonInteraction<'a> = (
    Entity,
    &'a Interaction,
    &'a mut Handle<ColorMaterial>,
    &'a Children,
);

fn click_play_button(
    mut commands: Commands,
    button_materials: Res<ButtonMaterials>,
    mut state: ResMut<State<GameState>>,
    mut interaction_query: Query<ButtonInteraction, (Changed<Interaction>, With<Button>)>,
    text_query: Query<Entity, With<Text>>,
    menu_query: Query<Entity, With<Menu>>,
) {
    for (button, interaction, mut material, children) in interaction_query.iter_mut() {
        let text = text_query.get(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                for entity in menu_query.iter() {
                    commands.entity(entity).despawn();
                }
                commands.entity(button).despawn();
                commands.entity(text).despawn();
                state.set(GameState::Playing).unwrap();
            }
            Interaction::Hovered => {
                *material = button_materials.hovered.clone();
            }
            Interaction::None => {
                *material = button_materials.normal.clone();
            }
        }
    }
}
