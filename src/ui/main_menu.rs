use bevy::{app::AppExit, ecs::system::SystemParam, input::keyboard::KeyboardInput, prelude::*};
use bevy_egui::{egui::style::Margin, *};
use bevy_fluent::Localization;
use iyes_loopless::state::NextState;
use leafwing_input_manager::{prelude::ActionState, user_input::InputKind};

use crate::{
    config::EngineConfig,
    input::MenuAction,
    metadata::{localization::LocalizationExt, ButtonStyle, FontStyle, GameMeta, Settings},
    platform::Storage,
    GameState,
};

use super::{
    widgets::{bordered_button::BorderedButton, bordered_frame::BorderedFrame, EguiUIExt},
    EguiContextExt, EguiResponseExt, WidgetAdjacencies,
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

#[derive(SystemParam)]
pub struct MenuSystemParams<'w, 's> {
    menu_page: Local<'s, MenuPage>,
    modified_settings: Local<'s, Option<Settings>>,
    currently_binding_input_idx: Local<'s, Option<usize>>,
    commands: Commands<'w, 's>,
    game: Res<'w, GameMeta>,
    localization: Res<'w, Localization>,
    engine_config: Res<'w, EngineConfig>,
    menu_input: Query<'w, 's, &'static ActionState<MenuAction>>,
    app_exit: EventWriter<'w, 's, AppExit>,
    storage: ResMut<'w, Storage>,
    adjacencies: ResMut<'w, WidgetAdjacencies>,
    control_inputs: ControlInputEvents<'w, 's>,
}

#[derive(SystemParam)]
pub struct ControlInputEvents<'w, 's> {
    keys: EventReader<'w, 's, KeyboardInput>,
    gamepad_events: EventReader<'w, 's, GamepadEvent>,
}

impl<'w, 's> ControlInputEvents<'w, 's> {
    // Get the next event, if any
    fn get_event(&mut self) -> Option<InputKind> {
        if let Some(event) = self.keys.iter().next() {
            event.key_code.map(Into::into)
        } else {
            None
        }
    }
}

/// Render the main menu UI
pub fn main_menu_system(mut params: MenuSystemParams, mut egui_context: ResMut<EguiContext>) {
    let menu_input = params.menu_input.single();

    // Go to previous menu if back button is pressed
    if menu_input.pressed(MenuAction::Back) {
        if let MenuPage::Settings { .. } = *params.menu_page {
            *params.menu_page = MenuPage::Main;
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

            // Create menu panel
            BorderedFrame::new(&params.game.ui_theme.panel.border)
                .margin(outer_margin)
                .padding(params.game.ui_theme.panel.padding.into())
                .show(ui, |ui| {
                    // Make sure the frame ocupies the entire rect that we allocated for it.
                    //
                    // Without this it would only take up enough size to fit it's content.
                    ui.set_min_size(ui.available_size());

                    // Render the menu based on the current menu selection
                    match *params.menu_page {
                        MenuPage::Main => main_menu_ui(&mut params, ui),
                        MenuPage::Settings { tab } => settings_menu_ui(&mut params, ui, tab),
                    }
                });
        });
}

/// Render the main menu
fn main_menu_ui(params: &mut MenuSystemParams, ui: &mut egui::Ui) {
    let MenuSystemParams {
        menu_page,
        modified_settings,
        commands,
        game,
        localization,
        engine_config,
        app_exit,
        storage,
        ..
    } = params;

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
        .show(ui)
        .fous_by_default(ui);

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
            **menu_page = MenuPage::Settings { tab: default() };
            **modified_settings = Some(
                storage
                    .get(Settings::STORAGE_KEY)
                    .unwrap_or_else(|| game.default_settings.clone()),
            );
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

/// Render the settings menu
fn settings_menu_ui(params: &mut MenuSystemParams, ui: &mut egui::Ui, current_tab: SettingsTab) {
    ui.vertical_centered(|ui| {
        // Settings Heading
        ui.themed_label(
            params
                .game
                .ui_theme
                .font_styles
                .get(&FontStyle::Heading)
                .unwrap(),
            &params.localization.get("settings"),
        );

        // Add tab list at the top of the panel
        let mut tabs = Vec::new();
        ui.horizontal(|ui| {
            for (i, (tab, name)) in SettingsTab::TABS.iter().enumerate() {
                let name = &params.localization.get(*name);
                let mut name = egui::RichText::new(name);

                if tab == &current_tab {
                    name = name.underline();
                }

                let mut button =
                    BorderedButton::themed(&params.game.ui_theme, &ButtonStyle::Normal, name)
                        .show(ui);

                if i == 0 {
                    button = button.fous_by_default(ui);
                }

                if button.clicked() {
                    *params.menu_page = MenuPage::Settings { tab: *tab };
                }

                tabs.push(button);
            }
        });

        // Add buttons to the bottom
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            let save_button = ui
                .horizontal(|ui| {
                    // Calculate button size and spacing
                    let width = ui.available_width();
                    let button_width = 0.3 * width;
                    let button_min_size = egui::vec2(button_width, 0.0);
                    let button_spacing = (width - 2.0 * button_width) / 3.0;

                    ui.add_space(button_spacing);

                    // Cancel button
                    let cancel_button = BorderedButton::themed(
                        &params.game.ui_theme,
                        &ButtonStyle::Normal,
                        &params.localization.get("cancel"),
                    )
                    .min_size(button_min_size)
                    .show(ui);

                    if cancel_button.clicked() {
                        *params.menu_page = MenuPage::Main;
                        ui.ctx().clear_focus();
                    }

                    ui.add_space(button_spacing);

                    // Save button
                    let save_button = BorderedButton::themed(
                        &params.game.ui_theme,
                        &ButtonStyle::Normal,
                        &params.localization.get("save"),
                    )
                    .min_size(button_min_size)
                    .show(ui);

                    if save_button.clicked() {
                        // Save the new settings
                        params.storage.set(
                            Settings::STORAGE_KEY,
                            params.modified_settings.as_ref().unwrap(),
                        );
                        params.storage.save();

                        *params.menu_page = MenuPage::Main;
                        ui.ctx().clear_focus();
                    }

                    // Return the save button so that controls settings can use it's response for
                    // adjacency.
                    save_button
                })
                .inner;

            ui.vertical(|ui| {
                // Render selected tab
                match current_tab {
                    SettingsTab::Controls => controls_settings_ui(params, ui, &tabs, save_button),
                    SettingsTab::Sound => sound_settings_ui(ui, &params.game),
                }
            });
        });
    });
}

// Render the control input grid
fn controls_settings_ui(
    params: &mut MenuSystemParams,
    ui: &mut egui::Ui,
    settings_tabs: &[egui::Response],
    save_button: egui::Response,
) {
    use egui_extras::Size;

    let ui_theme = &params.game.ui_theme;

    // Get the font meta for the table headings and labels
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

    // Calculate the row height so that it can fit the buttons
    let small_button_style = ui_theme.button_styles.get(&ButtonStyle::Small).unwrap();
    let row_height = small_button_style.font.size
        + small_button_style.padding.top
        + small_button_style.padding.bottom;

    // Mutably borrow the player controlls settings
    let controls = &mut params.modified_settings.as_mut().unwrap().player_controls;

    // Build the grid of control bindings as an array of mutable InputKind's
    let mut input_rows = [
        (
            &params.localization.get("move-up"),
            [
                &mut controls.keyboard1.movement.up,
                &mut controls.keyboard2.movement.up,
                &mut controls.gamepad.movement.up,
            ],
        ),
        (
            &params.localization.get("move-down"),
            [
                &mut controls.keyboard1.movement.down,
                &mut controls.keyboard2.movement.down,
                &mut controls.gamepad.movement.down,
            ],
        ),
        (
            &params.localization.get("move-left"),
            [
                &mut controls.keyboard1.movement.left,
                &mut controls.keyboard2.movement.left,
                &mut controls.gamepad.movement.left,
            ],
        ),
        (
            &params.localization.get("move-right"),
            [
                &mut controls.keyboard1.movement.right,
                &mut controls.keyboard2.movement.right,
                &mut controls.gamepad.movement.right,
            ],
        ),
        (
            &params.localization.get("flop-attack"),
            [
                &mut controls.keyboard1.flop_attack,
                &mut controls.keyboard2.flop_attack,
                &mut controls.gamepad.flop_attack,
            ],
        ),
        (
            &params.localization.get("shoot"),
            [
                &mut controls.keyboard1.shoot,
                &mut controls.keyboard2.shoot,
                &mut controls.gamepad.shoot,
            ],
        ),
        (
            &params.localization.get("throw"),
            [
                &mut controls.keyboard1.throw,
                &mut controls.keyboard2.throw,
                &mut controls.gamepad.throw,
            ],
        ),
    ];

    // Collect input button responses for building adjacency graph
    let mut input_buttons = Vec::new();

    // Create input table
    egui_extras::TableBuilder::new(ui)
        .cell_layout(egui::Layout::centered_and_justified(
            egui::Direction::LeftToRight,
        ))
        .column(Size::exact(label_font.size * 7.0))
        .column(Size::remainder())
        .column(Size::remainder())
        .column(Size::remainder())
        .header(bigger_font.size * 1.5, |mut row| {
            row.col(|ui| {
                ui.themed_label(&bigger_font, &params.localization.get("action"));
            });
            row.col(|ui| {
                ui.themed_label(&bigger_font, &params.localization.get("keyboard-1"));
            });
            row.col(|ui| {
                ui.themed_label(&bigger_font, &params.localization.get("keyboard-2"));
            });
            row.col(|ui| {
                ui.themed_label(&bigger_font, &params.localization.get("gamepad"));
            });
        })
        .body(|mut body| {
            // Loop through the input rows
            let mut input_idx = 0;
            for (title, inputs) in &mut input_rows {
                body.row(row_height, |mut row| {
                    // Add row label
                    row.col(|ui| {
                        ui.themed_label(&label_font, title);
                    });

                    // Add buttons for each kind of input
                    for input in inputs {
                        row.col(|ui| {
                            ui.set_enabled(params.currently_binding_input_idx.is_none());

                            let button = BorderedButton::themed(
                                ui_theme,
                                &ButtonStyle::Small,
                                format_input(input),
                            )
                            .show(ui);

                            if button.clicked() {
                                *params.currently_binding_input_idx = Some(input_idx);
                            }

                            if *params.currently_binding_input_idx == Some(input_idx) {
                                egui::Window::new("overlay")
                                    .auto_sized()
                                    .collapsible(false)
                                    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                                    .frame(egui::Frame::none())
                                    .title_bar(false)
                                    .show(ui.ctx(), |ui| {
                                        ui.label("Press something");

                                        if let Some(input_kind) = params.control_inputs.get_event()
                                        {
                                            *params.currently_binding_input_idx = None;
                                            **input = input_kind;
                                        }
                                    });
                            }

                            input_buttons.push(button);
                        });

                        input_idx += 1;
                    }
                });
            }
        });

    // Set adjacency for all of the gamepad input buttons
    for row_idx in 0..input_rows.len() {
        if row_idx == 0 {
            // Reverse button order here so that the first button gets priority when navigating down
            // from the tabs.
            for i in (0..3).rev() {
                let button = &input_buttons[row_idx * 3 + i];

                // The top row of buttons is below the settings tabs
                for tab in settings_tabs {
                    params.adjacencies.widget(tab).above(button);
                }
            }

        // If this is the last row, the buttons are above the save button
        } else if row_idx == input_rows.len() - 1 {
            for i in 0..3 {
                let button_above = &input_buttons[(row_idx - 1) * 3 + i];
                let button = &input_buttons[row_idx * 3 + i];

                params
                    .adjacencies
                    .widget(button)
                    .above(&save_button)
                    .below(button_above);
            }

        // If this is a middle row, set the buttons to be below the ones in the row above
        } else {
            for i in 0..3 {
                let button_above = &input_buttons[(row_idx - 1) * 3 + i];
                let button = &input_buttons[row_idx * 3 + i];

                params.adjacencies.widget(button).below(button_above);
            }
        }
    }
}

/// Render the sound settings
fn sound_settings_ui(ui: &mut egui::Ui, game: &GameMeta) {
    let ui_theme = &game.ui_theme;

    let font = ui_theme
        .font_styles
        .get(&FontStyle::Heading)
        .unwrap()
        .colored(ui_theme.panel.font_color);

    ui.centered_and_justified(|ui| ui.themed_label(&font, "Coming Soon!"));
}

fn format_input(input: &InputKind) -> String {
    match input {
        InputKind::SingleAxis(axis) => {
            // If we set the positive low to 1.0, then that means we don't trigger on positive
            // movement, and it must be a negative movement binding.
            let direction = if axis.positive_low == 1.0 { "-" } else { "+" };

            let stick = match axis.axis_type {
                leafwing_input_manager::axislike::AxisType::Gamepad(axis) => format!("{:?}", axis),
                other => format!("{:?}", other),
            };

            format!("{} {}", stick, direction)
        }
        other => other.to_string(),
    }
}
