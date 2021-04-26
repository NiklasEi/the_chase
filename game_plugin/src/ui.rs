use crate::loading::FontAssets;
use crate::map::{Acorn, Map};
use crate::menu::ButtonMaterials;
use crate::{GameData, GameState};
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<WonEvent>().add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(spawn_retry_ui.system())
                .with_system(click_retry_button.system()),
        );
    }
}

pub struct WonEvent;

struct Ui;
struct RetryButton;

fn spawn_retry_ui(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    button_materials: Res<ButtonMaterials>,
    mut won_events: EventReader<WonEvent>,
) {
    if won_events.iter().last().is_some() {
        commands
            .spawn_bundle(ButtonBundle {
                style: Style {
                    size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                    margin: Rect {
                        right: Val::Auto,
                        left: Val::Auto,
                        top: Val::Auto,
                        bottom: Val::Percent(20.),
                    },
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                material: button_materials.normal.clone(),
                ..Default::default()
            })
            .insert(RetryButton)
            .insert(Ui)
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: "Again!".to_string(),
                            style: TextStyle {
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                                font: font_assets.fira_sans.clone(),
                                ..Default::default()
                            },
                        }],
                        alignment: Default::default(),
                    },
                    ..Default::default()
                });
            });
    }
}

fn click_retry_button(
    mut commands: Commands,
    button_materials: Res<ButtonMaterials>,
    mut game_data: ResMut<GameData>,
    mut current_map: ResMut<Map>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>),
        (Changed<Interaction>, With<Button>),
    >,
    text_query: Query<Entity, (With<Ui>, Without<Acorn>)>,
    acorn: Query<Entity, (With<Acorn>, Without<Ui>)>,
) {
    for (interaction, mut material) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                game_data.won = false;
                *current_map = Map::Ground;
                for entity in acorn.iter() {
                    commands.entity(entity).despawn();
                }
                for entity in text_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }
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
