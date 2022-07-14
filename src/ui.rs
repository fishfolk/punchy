use bevy::{prelude::*, utils::HashMap, window::WindowId};
use bevy_egui::{egui, EguiContext, EguiInput, EguiPlugin, EguiSettings};
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{assets::EguiFont, audio::*, input::MenuAction, metadata::GameMeta, GameState};

pub mod hud;
pub mod widgets;

pub mod main_menu;
pub mod pause_menu;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .add_system(handle_menu_input.run_if_resource_exists::<GameMeta>())
            .add_enter_system(GameState::MainMenu, main_menu::spawn_main_menu_background)
            .add_enter_system(GameState::MainMenu, play_menu_music)
            .add_exit_system(GameState::MainMenu, main_menu::despawn_main_menu_background)
            .add_exit_system(GameState::MainMenu, stop_menu_music)
            .add_system(hud::render_hud.run_in_state(GameState::InGame))
            .add_system(update_egui_fonts)
            .add_system(update_ui_scale.run_if_resource_exists::<GameMeta>())
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Paused)
                    .with_system(pause_menu::pause_menu)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::MainMenu)
                    .with_system(main_menu::main_menu_system)
                    .into(),
            );
    }
}

/// Extension trait with helpers the egui context and UI types
pub trait EguiUiExt {
    /// Clear the UI focus
    fn clear_focus(self);
}

impl EguiUiExt for &egui::Context {
    fn clear_focus(self) {
        self.memory().request_focus(egui::Id::null());
    }
}

impl EguiUiExt for &mut egui::Ui {
    fn clear_focus(self) {
        self.ctx().clear_focus();
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

    if input.just_pressed(MenuAction::Next) {
        events.push(egui::Event::Key {
            key: egui::Key::Tab,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        });
    }

    if input.just_pressed(MenuAction::Previous) {
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
    mut font_queue: Local<Vec<Handle<EguiFont>>>,
    mut egui_ctx: ResMut<EguiContext>,
    egui_font_definitions: Option<ResMut<egui::FontDefinitions>>,
    game: Option<Res<GameMeta>>,
    mut events: EventReader<AssetEvent<EguiFont>>,
    assets: Res<Assets<EguiFont>>,
) {
    // Add any newly updated/created fonts to the queue
    for event in events.iter() {
        if let AssetEvent::Created { handle } | AssetEvent::Modified { handle } = event {
            font_queue.push(handle.clone_weak());
        }
    }

    // Update queued fonts if the game is ready
    if let Some((game, mut egui_font_definitions)) = game.zip(egui_font_definitions) {
        for handle in font_queue.drain(..) {
            // Get the game font name associated to this handle
            let name = game
                .ui_theme
                .font_handles
                .iter()
                .find_map(|(font_name, font_handle)| {
                    if font_handle == &handle {
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
