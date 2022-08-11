use bevy::prelude::*;
use bevy_egui::*;
use bevy_fluent::Localization;
use bevy_inspector_egui::WorldInspectorParams;
use bevy_rapier2d::prelude::DebugRenderContext;

use crate::localization::LocalizationExt;

/// System that renders the debug tools window which can be toggled by pressing F12
pub fn debug_tools_window(
    mut visible: Local<bool>,
    mut egui_context: ResMut<EguiContext>,
    localization: Res<Localization>,
    input: Res<Input<KeyCode>>,
    mut rapier_debug: ResMut<DebugRenderContext>,
    mut inspector: ResMut<WorldInspectorParams>,
) {
    let ctx = egui_context.ctx_mut();

    // Toggle window visibility
    if input.just_pressed(KeyCode::F12) {
        *visible = !*visible;
    }

    // Display debug tool window
    egui::Window::new(localization.get("debug-tools"))
        // ID is needed because title comes from localizaition which can change
        .id(egui::Id::new("debug_tools"))
        .open(&mut *visible)
        .show(ctx, |ui| {
            // Show collision shapes
            ui.checkbox(
                &mut rapier_debug.enabled,
                localization.get("show-collision-shapes"),
            );

            // Show world inspector
            ui.checkbox(
                &mut inspector.enabled,
                localization.get("show-world-inspector"),
            );
        });
}
