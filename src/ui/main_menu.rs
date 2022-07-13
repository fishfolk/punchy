use bevy::prelude::*;
use bevy_egui::{egui::style::Margin, *};
use bevy_fluent::Localization;
use iyes_loopless::state::NextState;

use crate::{
    config::EngineConfig,
    metadata::{localization::LocalizationExt, ButtonStyle, GameMeta},
    GameState,
};

use super::widgets::{bordered_button::BorderedButton, bordered_frame::BorderedFrame, EguiUIExt};

#[derive(Component)]
pub struct MainMenuBackground;

/// Spawns the background image for the main menu
pub fn spawn_main_menu_background(
    mut commands: Commands,
    game: Res<GameMeta>,
    windows: Res<Windows>,
) {
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
pub fn despawn_main_menu_background(
    mut commands: Commands,
    background: Query<Entity, With<MainMenuBackground>>,
) {
    let bg = background.single();
    commands.entity(bg).despawn();
}

/// Render the main menu UI
pub fn main_menu_ui(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    game: Res<GameMeta>,
    localization: Res<Localization>,
    engine_config: Res<EngineConfig>,
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

                            if start_button.clicked() || engine_config.auto_start {
                                commands.insert_resource(game.start_level_handle.clone());
                                commands.insert_resource(NextState(GameState::LoadingLevel));
                            }
                        });
                    });
                })
        });
}
