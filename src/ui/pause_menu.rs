use bevy::prelude::*;
use bevy_egui::*;
use bevy_fluent::Localization;
use iyes_loopless::state::NextState;

use crate::{
    metadata::{localization::LocalizationExt, ButtonStyle, FontStyle, GameMeta},
    GameState,
};

use super::{
    widgets::{bordered_button::BorderedButton, bordered_frame::BorderedFrame, EguiUIExt},
    EguiContextExt,
};

pub fn pause_menu(
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
                            ui.ctx().clear_focus();
                        }
                    });
                })
        });
}
