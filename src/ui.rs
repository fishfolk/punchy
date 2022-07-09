use bevy::{prelude::*, utils::HashMap, window::WindowId};
use bevy_egui::{
    egui::{self, style::Margin},
    EguiContext, EguiInput, EguiPlugin, EguiSettings, EguiSystem,
};
use bevy_fluent::Localization;
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    assets::EguiFont,
    input::MenuAction,
    metadata::{localization::LocalizationExt, ButtonStyle, FontStyle, GameMeta},
    GameState,
};

use self::widgets::{bordered_button::BorderedButton, bordered_frame::BorderedFrame, EguiUIExt};

pub mod hud;
pub mod widgets;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .add_system(
                handle_menu_input
                    .run_if_resource_exists::<GameMeta>()
                    .after(EguiSystem::ProcessInput)
                    .before(EguiSystem::BeginFrame),
            )
            .add_enter_system(GameState::MainMenu, spawn_main_menu_background)
            .add_exit_system(GameState::MainMenu, despawn_main_menu_background)
            .add_system(hud::render_hud.run_in_state(GameState::InGame))
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

fn handle_menu_input(
    mut windows: ResMut<Windows>,
    input: Query<&ActionState<MenuAction>>,
    mut egui_inputs: ResMut<HashMap<WindowId, EguiInput>>,
) {
    use bevy::window::WindowMode;
    let input = input.single();

    // Handle fullscreen toggling
    if input.just_pressed(MenuAction::ToggleFullscreen) {
        if let Some(window) = windows.get_primary_mut() {
            window.set_mode(match window.mode() {
                WindowMode::BorderlessFullscreen => WindowMode::Windowed,
                _ => WindowMode::BorderlessFullscreen,
            });
        }
    }

    // Emit tab / shift + tab Egui events in response to menu navigation inputs. This is pretty
    // hacky and may need to be re-visited.
    let events = &mut egui_inputs
        .get_mut(&WindowId::primary())
        .unwrap()
        .raw_input
        .events;

    if input.just_pressed(MenuAction::Confirm) {
        events.push(egui::Event::Key {
            key: egui::Key::Enter,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        });
    }

    if input.just_pressed(MenuAction::Forward) {
        events.push(egui::Event::Key {
            key: egui::Key::Tab,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        });
    }

    if input.just_pressed(MenuAction::Backward) {
        events.push(egui::Event::Key {
            key: egui::Key::Tab,
            pressed: true,
            modifiers: egui::Modifiers::SHIFT,
        });
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

fn pause_menu(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    game: Res<GameMeta>,
    non_camera_entities: Query<Entity, Without<Camera>>,
    mut camera_transform: Query<&mut Transform, With<Camera>>,
    localization: Res<Localization>,
) {
    let ui_theme = &game.ui_theme;

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(egui_context.ctx_mut(), |ui| {
            let screen_rect = ui.max_rect();

            let pause_menu_width = 300.0;
            let x_margin = (screen_rect.width() - pause_menu_width) / 2.0;
            let outer_margin = egui::style::Margin::symmetric(x_margin, screen_rect.height() * 0.2);

            BorderedFrame::new(&ui_theme.panel.border)
                .margin(outer_margin)
                .padding(ui_theme.panel.padding.into())
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());

                    let heading_font = ui_theme
                        .font_styles
                        .get(&FontStyle::Heading)
                        .expect("Missing 'heading' font style")
                        .colored(ui_theme.panel.font_color);

                    ui.vertical_centered(|ui| {
                        ui.themed_label(&heading_font, &localization.get("paused"));

                        ui.add_space(10.0);

                        let width = ui.available_width();

                        let continue_button = BorderedButton::themed(
                            ui_theme,
                            &ButtonStyle::Normal,
                            &localization.get("continue"),
                        )
                        .min_size(egui::vec2(width, 0.0))
                        .show(ui);

                        // Focus continue button by default
                        if ui.memory().focus().is_none() {
                            continue_button.request_focus();
                        }

                        if continue_button.clicked() {
                            commands.insert_resource(NextState(GameState::InGame));
                        }

                        if BorderedButton::themed(
                            ui_theme,
                            &ButtonStyle::Normal,
                            &localization.get("main-menu"),
                        )
                        .min_size(egui::vec2(width, 0.0))
                        .show(ui)
                        .clicked()
                        {
                            // Clean up all entities other than the camera
                            for entity in non_camera_entities.iter() {
                                commands.entity(entity).despawn();
                            }
                            // Reset camera position
                            let mut camera_transform = camera_transform.single_mut();
                            camera_transform.translation.x = 0.0;
                            camera_transform.translation.y = 0.0;

                            // Show the main menu
                            commands.insert_resource(NextState(GameState::MainMenu));
                        }
                    });
                })
        });
}

#[derive(Component)]
struct MainMenuBackground;

/// Spawns the background image for the main menu
fn spawn_main_menu_background(mut commands: Commands, game: Res<GameMeta>, windows: Res<Windows>) {
    let window = windows.primary();
    let bg_handle = game.main_menu.background_image.image_handle.clone();
    let img_size = game.main_menu.background_image.image_size;
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
fn main_menu(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    game: Res<GameMeta>,
    localization: Res<Localization>,
) {
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
                    // Make sure the frame ocupies the entire rect that we allocated for it.
                    //
                    // Without this it would only take up enough size to fit it's content.
                    ui.set_min_size(ui.available_size());

                    // Create a vertical list of items, centered horizontally
                    ui.vertical_centered(|ui| {
                        ui.themed_label(&game.main_menu.title_font, &localization.get("title"));

                        // Now switch the layout to bottom_up so that we can start adding widgets
                        // from the bottom of the frame.
                        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                            let start_button = BorderedButton::themed(
                                ui_theme,
                                &ButtonStyle::Jumbo,
                                &localization.get("start-game"),
                            )
                            .show(ui);

                            // Focus the start button if nothing else is focused. That way you can
                            // play the game just by pressing Enter.
                            if ui.memory().focus().is_none() {
                                start_button.request_focus();
                            }

                            if start_button.clicked() {
                                commands.insert_resource(game.start_level_handle.clone());
                                commands.insert_resource(NextState(GameState::LoadingLevel));
                            }
                        });
                    });
                })
        });
}
