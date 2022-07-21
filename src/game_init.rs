use bevy::{ecs::system::SystemParam, prelude::*, render::camera::ScalingMode};
use bevy_egui::{egui, EguiContext};
use bevy_fluent::Locale;
use bevy_parallax::ParallaxCameraComponent;
use iyes_loopless::state::NextState;
use leafwing_input_manager::{
    axislike::{AxisType, SingleAxis},
    prelude::InputMap,
    InputManagerBundle,
};

use crate::{
    input::MenuAction,
    metadata::{BorderImageMeta, GameMeta},
    GameState,
};

#[derive(SystemParam)]
pub struct GameLoader<'w, 's> {
    skip_next_asset_update_event: Local<'s, bool>,
    camera: Query<'w, 's, Entity, With<Camera>>,
    commands: Commands<'w, 's>,
    game_handle: Res<'w, Handle<GameMeta>>,
    assets: ResMut<'w, Assets<GameMeta>>,
    egui_ctx: ResMut<'w, EguiContext>,
    asset_server: Res<'w, AssetServer>,
    events: EventReader<'w, 's, AssetEvent<GameMeta>>,
}

/// System to run the initial game load
pub fn load_game(loader: GameLoader) {
    loader.load(false);
}

/// System to check for asset changes and hot reload the game
pub fn hot_reload_game(loader: GameLoader) {
    loader.load(true);
}

impl<'w, 's> GameLoader<'w, 's> {
    /// This function is called once when the game starts up and, when hot reload is enabled, on
    /// update, to check for asset changed events and to update the [`GameMeta`] resource.
    ///
    /// The `is_hot_reload` argument is used to indicate whether the function should check for asset
    /// updates and reload, or whether it should run the one-time initialization of the game.
    fn load(mut self, is_hot_reload: bool) {
        // Check to make sure we shouldn't skip this execution
        // ( i.e. if this is a hot reload run without any changed assets )
        if self.should_skip_run(is_hot_reload) {
            return;
        }

        let Self {
            mut skip_next_asset_update_event,
            camera,
            mut commands,
            game_handle,
            mut assets,
            mut egui_ctx,
            asset_server,
            ..
        } = self;

        if let Some(game) = assets.get_mut(game_handle.clone_weak()) {
            debug!("Loaded game");

            // Hot reload preparation
            if is_hot_reload {
                // Despawn previous camera
                if let Ok(camera) = camera.get_single() {
                    commands.entity(camera).despawn();
                }

                // Since we are modifying the game asset, which will trigger another asset changed
                // event, we need to skip the next update event.
                *skip_next_asset_update_event = true;

            // One-time initialization
            } else {
                // Initialize empty fonts for all game fonts.
                //
                // This makes sure Egui will not panic if we try to use a font that is still loading.
                let mut egui_fonts = egui::FontDefinitions::default();
                for font_name in game.ui_theme.font_families.keys() {
                    let font_family = egui::FontFamily::Name(font_name.clone().into());
                    egui_fonts.families.insert(font_family, vec![]);
                }
                egui_ctx.ctx_mut().set_fonts(egui_fonts.clone());
                commands.insert_resource(egui_fonts);

                // Transition to the main menu when we are done
                commands.insert_resource(NextState(GameState::MainMenu));
            }

            // Set the locale resource
            let translations = &game.translations;
            commands.insert_resource(
                Locale::new(translations.detected_locale.clone())
                    .with_default(translations.default_locale.clone()),
            );

            // Spawn the camera
            let mut camera_bundle = OrthographicCameraBundle::new_2d();
            // camera_bundle.orthographic_projection.depth_calculation = DepthCalculation::Distance;
            camera_bundle.orthographic_projection.scaling_mode = ScalingMode::FixedVertical;
            camera_bundle.orthographic_projection.scale = game.camera_height as f32 / 2.0;
            commands
                .spawn_bundle(camera_bundle)
                .insert(ParallaxCameraComponent)
                // We also add another input manager bundle for `MenuAction`s
                .insert_bundle(InputManagerBundle {
                    input_map: menu_input_map(),
                    ..default()
                });

            // Helper to load border images
            let mut load_border_image = |border: &mut BorderImageMeta| {
                border.handle = asset_server.load(&border.image);
                border.egui_texture = egui_ctx.add_image(border.handle.clone_weak());
            };

            // Load border images
            load_border_image(&mut game.ui_theme.hud.portrait_frame);
            load_border_image(&mut game.ui_theme.panel.border);
            load_border_image(&mut game.ui_theme.hud.lifebar.background_image);
            load_border_image(&mut game.ui_theme.hud.lifebar.progress_image);
            for button in game.ui_theme.button_styles.values_mut() {
                load_border_image(&mut button.borders.default);
                if let Some(border) = &mut button.borders.clicked {
                    load_border_image(border);
                }
                if let Some(border) = &mut button.borders.focused {
                    load_border_image(border);
                }
            }

            // Insert the game resource
            commands.insert_resource(game.clone());
            commands.insert_resource(game.start_level.clone());

        // If the game asset isn't loaded yet
        } else {
            trace!("Awaiting game load")
        }
    }

    // Run checks to see if we should skip running the system
    fn should_skip_run(&mut self, is_hot_reload: bool) -> bool {
        // If this is a hot reload run, check for modified asset events
        if is_hot_reload {
            let mut has_update = false;
            for (event, event_id) in self.events.iter_with_id() {
                if let AssetEvent::Modified { .. } = event {
                    // We may need to skip an asset update event
                    if *self.skip_next_asset_update_event {
                        *self.skip_next_asset_update_event = false;
                    } else {
                        debug!(%event_id, "Game updated");
                        has_update = true;
                    }
                }
            }

            // If there was no update, skip execution
            if !has_update {
                return true;
            }
        }

        false
    }
}

fn menu_input_map() -> InputMap<MenuAction> {
    InputMap::default()
        // Up
        .insert(KeyCode::Up, MenuAction::Up)
        .insert(GamepadButtonType::DPadUp, MenuAction::Up)
        .insert(
            SingleAxis {
                axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickY),
                positive_low: 0.5,
                negative_low: -1.0,
                value: None,
            },
            MenuAction::Up,
        )
        // Left
        .insert(KeyCode::Left, MenuAction::Left)
        .insert(GamepadButtonType::DPadLeft, MenuAction::Left)
        .insert(
            SingleAxis {
                axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickX),
                positive_low: 1.0,
                negative_low: -0.5,
                value: None,
            },
            MenuAction::Left,
        )
        // Down
        .insert(KeyCode::Down, MenuAction::Down)
        .insert(GamepadButtonType::DPadDown, MenuAction::Down)
        .insert(
            SingleAxis {
                axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickY),
                positive_low: 1.0,
                negative_low: -0.5,
                value: None,
            },
            MenuAction::Down,
        )
        // Right
        .insert(KeyCode::Right, MenuAction::Right)
        .insert(GamepadButtonType::DPadRight, MenuAction::Right)
        .insert(
            SingleAxis {
                axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickX),
                positive_low: 0.5,
                negative_low: -1.0,
                value: None,
            },
            MenuAction::Right,
        )
        // Confirm
        .insert(KeyCode::Return, MenuAction::Confirm)
        .insert(GamepadButtonType::South, MenuAction::Confirm)
        .insert(GamepadButtonType::Start, MenuAction::Confirm)
        // Back
        .insert(KeyCode::Escape, MenuAction::Back)
        .insert(GamepadButtonType::East, MenuAction::Back)
        // Toggle Fullscreen
        .insert(KeyCode::F11, MenuAction::ToggleFullscreen)
        .insert(GamepadButtonType::Mode, MenuAction::ToggleFullscreen)
        // Pause
        .insert(KeyCode::Escape, MenuAction::Pause)
        .insert(GamepadButtonType::Start, MenuAction::Pause)
        .build()
}
