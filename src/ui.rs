use bevy::prelude::*;
use bevy_egui::{
    egui::{self, style::Margin, RichText},
    EguiContext, EguiPlugin, EguiSettings,
};
use iyes_loopless::prelude::*;

use crate::{assets::EguiFont, metadata::GameMeta, GameState};

use self::widgets::{bordered_button::BorderedButton, bordered_frame::BorderedFrame};

pub mod widgets;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .add_enter_system(GameState::MainMenu, spawn_main_menu_background)
            .add_exit_system(GameState::MainMenu, despawn_main_menu_background)
            .add_system(update_egui_fonts.run_if_resource_exists::<GameMeta>())
            .add_system(update_ui_scale.run_if_resource_exists::<GameMeta>())
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Paused)
                    .with_system(pause_menu)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::MainMenu)
                    .with_system(main_menu)
                    .into(),
            );
    }
}

/// Watches for asset events for [`EguiFont`] assets and updates the corresponding fonts from the
/// [`GameMeta`], inserting the font data into the egui context.
fn update_egui_fonts(
    mut egui_font_definitions: ResMut<egui::FontDefinitions>,
    mut egui_ctx: ResMut<EguiContext>,
    game: Res<GameMeta>,
    mut events: EventReader<AssetEvent<EguiFont>>,
    assets: Res<Assets<EguiFont>>,
) {
    for event in events.iter() {
        if let AssetEvent::Created { handle } | AssetEvent::Modified { handle } = event {
            // Get the game font name associated to this handle
            let name = game
                .ui_theme
                .font_handles
                .iter()
                .find_map(|(font_name, font_handle)| {
                    if font_handle == handle {
                        Some(font_name.clone())
                    } else {
                        None
                    }
                });

            // If we were able to find the font handle in our game fonts
            if let Some(font_name) = name {
                // Get the font asset
                if let Some(font) = assets.get(handle) {
                    // And insert it into the Egui font definitions
                    let ctx = egui_ctx.ctx_mut();
                    egui_font_definitions
                        .font_data
                        .insert(font_name.clone(), font.0.clone());

                    egui_font_definitions
                        .families
                        .get_mut(&egui::FontFamily::Name(font_name.clone().into()))
                        .unwrap()
                        .push(font_name);

                    ctx.set_fonts(egui_font_definitions.clone());
                }
            }
        }
    }
}

/// This system makes sure that the UI scale of Egui matches our game scale so that a pixel in egui
/// will be the same size as a pixel in our sprites.
fn update_ui_scale(
    mut egui_settings: ResMut<EguiSettings>,
    windows: Res<Windows>,
    projection: Query<&OrthographicProjection, With<Camera>>,
) {
    if let Some(window) = windows.get_primary() {
        if let Ok(projection) = projection.get_single() {
            match projection.scaling_mode {
                bevy::render::camera::ScalingMode::FixedVertical => {
                    let window_height = window.height();
                    let scale = window_height / (projection.scale * 2.0);
                    egui_settings.scale_factor = scale as f64;
                }
                bevy::render::camera::ScalingMode::FixedHorizontal => {
                    let window_width = window.width();
                    let scale = window_width / (projection.scale * 2.0);
                    egui_settings.scale_factor = scale as f64;
                }
                bevy::render::camera::ScalingMode::None => (),
                bevy::render::camera::ScalingMode::WindowSize => (),
            }
        }
    }
}

fn pause_menu(mut egui_context: ResMut<EguiContext>) {
    egui::Window::new("Paused").show(egui_context.ctx_mut(), |_ui| {});
}

#[derive(Component)]
struct MainMenuBackground;

/// Spawns the background image for the main menu
fn spawn_main_menu_background(mut commands: Commands, game: Res<GameMeta>, windows: Res<Windows>) {
    let window = windows.primary();
    let bg_handle = game.main_menu.background_image.handle.clone();
    let img_size = game.main_menu.background_image.size;
    let ratio = img_size.x / img_size.y;
    let height = window.height();
    let width = height * ratio;
    commands
        .spawn_bundle(SpriteBundle {
            texture: bg_handle,
            sprite: Sprite {
                custom_size: Some(Vec2::new(width, height)),
                ..default()
            },
            ..default()
        })
        .insert(MainMenuBackground);
}

/// Despawns the background image for the main menu
fn despawn_main_menu_background(
    mut commands: Commands,
    background: Query<Entity, With<MainMenuBackground>>,
) {
    let bg = background.single();
    commands.entity(bg).despawn();
}

/// Render the main menu UI
fn main_menu(mut commands: Commands, mut egui_context: ResMut<EguiContext>, game: Res<GameMeta>) {
    let ui_theme = &game.ui_theme;

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(egui_context.ctx_mut(), |ui| {
            let screen_rect = ui.max_rect();

            // Calculate a margin of 20% of the screen size
            let outer_margin = screen_rect.size() * 0.20;
            let outer_margin = Margin {
                left: outer_margin.x,
                right: outer_margin.x,
                // Make top and bottom margins smaller
                top: outer_margin.y / 1.5,
                bottom: outer_margin.y / 1.5,
            };

            BorderedFrame::new(&ui_theme.panel.border)
                .margin(outer_margin)
                .padding(ui_theme.panel.padding.into())
                .show(ui, |ui| {
                    let text_color = ui_theme.panel.text_color;

                    // Make sure the frame ocupies the entire rect that we allocated for it.
                    //
                    // Without this it would only take up enough size to fit it's content.
                    ui.set_min_size(ui.available_size());

                    // Create a vertical list of items, centered horizontally
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(&game.main_menu.title)
                                .font(egui::FontId::new(
                                    game.main_menu.title_size,
                                    egui::FontFamily::Name(
                                        game.main_menu.title_font.clone().into(),
                                    ),
                                ))
                                .color(text_color),
                        );

                        // Now switch the layout to bottom_up so that we can start adding widgets
                        // from the bottom of the frame.
                        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                            if BorderedButton::new(
                                RichText::new("Start Game")
                                    .font(egui::FontId::new(
                                        ui_theme.button.font_size,
                                        egui::FontFamily::Name(ui_theme.button.font.clone().into()),
                                    ))
                                    .color(ui_theme.button.text_color),
                            )
                            .padding(ui_theme.button.padding.into())
                            .border(&ui_theme.button.borders.default)
                            .on_click_border(ui_theme.button.borders.clicked.as_ref())
                            .on_hover_border(ui_theme.button.borders.hovered.as_ref())
                            .show(ui)
                            .clicked()
                            {
                                commands.insert_resource(game.start_level_handle.clone());
                                commands.insert_resource(NextState(GameState::LoadingLevel));
                            }
                        });
                    });
                })
        });
}
