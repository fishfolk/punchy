use bevy::{app::AppExit, prelude::*};
use bevy_egui::{egui::style::Margin, *};
use bevy_fluent::Localization;
use iyes_loopless::state::NextState;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    config::{EngineConfig, Settings},
    input::MenuAction,
    metadata::{localization::LocalizationExt, ButtonStyle, FontStyle, GameMeta},
    platform::Storage,
    GameState,
};

use super::{
    widgets::{bordered_button::BorderedButton, bordered_frame::BorderedFrame, EguiUIExt},
    EguiUiExt,
};

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

#[derive(Clone, Copy)]
pub enum MenuPage {
    Main,
    Settings { tab: SettingsTab },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Controls,
    Sound,
}

impl Default for MenuPage {
    fn default() -> Self {
        Self::Main
    }
}

impl Default for SettingsTab {
    fn default() -> Self {
        Self::Controls
    }
}

impl SettingsTab {
    const TABS: &'static [(Self, &'static str)] =
        &[(Self::Controls, "controls"), (Self::Sound, "sound")];
}

/// Render the main menu UI
pub fn main_menu_system(
    mut menu_state: Local<MenuPage>,
    mut settings: Local<Settings>,
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    game: Res<GameMeta>,
    localization: Res<Localization>,
    engine_config: Res<EngineConfig>,
    menu_input: Query<&ActionState<MenuAction>>,
    mut app_exit: EventWriter<AppExit>,
    mut storage: ResMut<Storage>,
) {
    let menu_input = menu_input.single();

    // Go to previous menu if back button is pressed
    if menu_input.pressed(MenuAction::Back) {
        if let MenuPage::Settings { .. } = *menu_state {
            *menu_state = MenuPage::Main;
            egui_context.ctx_mut().clear_focus();
        }
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(egui_context.ctx_mut(), |ui| {
            let screen_rect = ui.max_rect();

            // Calculate a margin
            let outer_margin = screen_rect.size() * 0.10;
            let outer_margin = Margin {
                left: outer_margin.x,
                right: outer_margin.x,
                // Make top and bottom margins smaller
                top: outer_margin.y / 1.5,
                bottom: outer_margin.y / 1.5,
            };

            BorderedFrame::new(&game.ui_theme.panel.border)
                .margin(outer_margin)
                .padding(game.ui_theme.panel.padding.into())
                .show(ui, |ui| {
                    // Make sure the frame ocupies the entire rect that we allocated for it.
                    //
                    // Without this it would only take up enough size to fit it's content.
                    ui.set_min_size(ui.available_size());

                    // Render the menu based on the current menu selection
                    match *menu_state {
                        MenuPage::Main => main_menu_ui(
                            ui,
                            &mut settings,
                            &mut menu_state,
                            &mut commands,
                            &mut app_exit,
                            &mut storage,
                            &localization,
                            &game,
                            &engine_config,
                        ),
                        MenuPage::Settings { tab } => settings_menu_ui(
                            ui,
                            &mut settings,
                            &mut menu_state,
                            &mut storage,
                            tab,
                            &localization,
                            &game,
                        ),
                    }
                });
        });
}

fn main_menu_ui(
    ui: &mut egui::Ui,
    settings: &mut Settings,
    menu_page: &mut MenuPage,
    commands: &mut Commands,
    app_exit: &mut EventWriter<AppExit>,
    storage: &mut Storage,
    localization: &Localization,
    game: &GameMeta,
    engine_config: &EngineConfig,
) {
    let ui_theme = &game.ui_theme;

    // Create a vertical list of items, centered horizontally
    ui.vertical_centered(|ui| {
        ui.themed_label(&game.main_menu.title_font, &localization.get("title"));
        ui.add_space(game.main_menu.title_font.size);

        let min_button_size = egui::vec2(ui.available_width() / 2.0, 0.0);

        // Start button
        let start_button = BorderedButton::themed(
            ui_theme,
            &ButtonStyle::Normal,
            &localization.get("start-game"),
        )
        .min_size(min_button_size)
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

        // Settings button
        if BorderedButton::themed(
            ui_theme,
            &ButtonStyle::Normal,
            &localization.get("settings"),
        )
        .min_size(min_button_size)
        .show(ui)
        .clicked()
        {
            *menu_page = MenuPage::Settings { tab: default() };
            *settings = storage.get(Settings::STORAGE_KEY).unwrap_or_default();
        }

        // Quit button
        if BorderedButton::themed(ui_theme, &ButtonStyle::Normal, &localization.get("quit"))
            .min_size(min_button_size)
            .show(ui)
            .clicked()
        {
            app_exit.send(AppExit);
        }
    });
}

fn settings_menu_ui(
    ui: &mut egui::Ui,
    settings: &mut Settings,
    menu_page: &mut MenuPage,
    storage: &mut Storage,
    current_tab: SettingsTab,
    localization: &Localization,
    game: &GameMeta,
) {
    let ui_theme = &game.ui_theme;

    ui.vertical_centered(|ui| {
        // Settings Heading
        ui.themed_label(
            game.ui_theme.font_styles.get(&FontStyle::Heading).unwrap(),
            &localization.get("settings"),
        );

        // Add tab list to the top of the panel
        ui.horizontal(|ui| {
            for (tab, name) in SettingsTab::TABS {
                let name = &localization.get(*name);
                let mut name = egui::RichText::new(name);

                if tab == &current_tab {
                    name = name.underline();
                }

                if BorderedButton::themed(ui_theme, &ButtonStyle::Normal, name)
                    .show(ui)
                    .clicked()
                {
                    *menu_page = MenuPage::Settings { tab: *tab };
                }
            }
        });

        // Add buttons to the bottom
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.horizontal(|ui| {
                // Calculate button size and spacing
                let width = ui.available_width();
                let button_width = 0.3 * width;
                let button_min_size = egui::vec2(button_width, 0.0);
                let button_spacing = (width - 2.0 * button_width) / 3.0;

                ui.add_space(button_spacing);

                // Cancel button
                if BorderedButton::themed(
                    ui_theme,
                    &ButtonStyle::Normal,
                    &localization.get("cancel"),
                )
                .min_size(button_min_size)
                .show(ui)
                .clicked()
                {
                    *menu_page = MenuPage::Main;
                    ui.clear_focus();
                }

                ui.add_space(button_spacing);

                // Save button
                if BorderedButton::themed(ui_theme, &ButtonStyle::Normal, &localization.get("save"))
                    .min_size(button_min_size)
                    .show(ui)
                    .clicked()
                {
                    // Save the new settings
                    storage.set(Settings::STORAGE_KEY, settings);
                    storage.save();

                    *menu_page = MenuPage::Main;
                    ui.clear_focus();
                }
            });

            ui.vertical(|ui| {
                // Render selected tab
                match current_tab {
                    SettingsTab::Controls => controls_settings_ui(ui, game),
                    SettingsTab::Sound => sound_settings_ui(ui, game),
                }
            });
        });
    });
}

fn controls_settings_ui(ui: &mut egui::Ui, game: &GameMeta) {
    use egui_extras::Size;

    let ui_theme = &game.ui_theme;

    let bigger_font = ui_theme
        .font_styles
        .get(&FontStyle::Bigger)
        .unwrap()
        .colored(ui_theme.panel.font_color);
    let label_font = ui_theme
        .font_styles
        .get(&FontStyle::Normal)
        .unwrap()
        .colored(ui_theme.panel.font_color);

    ui.add_space(2.0);

    let row_height = label_font.size * 1.5;

    egui_extras::TableBuilder::new(ui)
        .cell_layout(egui::Layout::left_to_right().with_cross_align(egui::Align::Center))
        .column(Size::exact(label_font.size * 7.0))
        .column(Size::remainder())
        .column(Size::remainder())
        .column(Size::remainder())
        .header(bigger_font.size, |mut row| {
            row.col(|ui| {
                ui.themed_label(&bigger_font, "Action");
            });
            row.col(|ui| {
                ui.themed_label(&bigger_font, "Keyboard 1");
            });
            row.col(|ui| {
                ui.themed_label(&bigger_font, "Keyboard 2");
            });
            row.col(|ui| {
                ui.themed_label(&bigger_font, "Gampead");
            });
        })
        .body(|mut body| {
            body.row(row_height, |mut row| {
                row.col(|ui| {
                    ui.themed_label(&label_font, "Move Up");
                });
            });
            body.row(row_height, |mut row| {
                row.col(|ui| {
                    ui.themed_label(&label_font, "Move Down");
                });
            });
            body.row(row_height, |mut row| {
                row.col(|ui| {
                    ui.themed_label(&label_font, "Move Left");
                });
            });
            body.row(row_height, |mut row| {
                row.col(|ui| {
                    ui.themed_label(&label_font, "Move Right");
                });
            });
            body.row(row_height, |mut row| {
                row.col(|ui| {
                    ui.themed_label(&label_font, "Flop Attack");
                });
            });
            body.row(row_height, |mut row| {
                row.col(|ui| {
                    ui.themed_label(&label_font, "Throw");
                });
            });
            body.row(row_height, |mut row| {
                row.col(|ui| {
                    ui.themed_label(&label_font, "Shoot");
                });
            });
        });
}

fn sound_settings_ui(ui: &mut egui::Ui, game: &GameMeta) {
    let ui_theme = &game.ui_theme;

    let font = ui_theme
        .font_styles
        .get(&FontStyle::Heading)
        .unwrap()
        .colored(ui_theme.panel.font_color);

    ui.centered_and_justified(|ui| {
        ui.themed_label(&font, "Coming Soon!")
    });
}
