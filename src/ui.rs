use bevy::prelude::{App, Component, Plugin, Query, ResMut};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use iyes_loopless::condition::ConditionSet;

use crate::GameState;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
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

fn pause_menu(mut egui_context: ResMut<EguiContext>) {
    egui::Window::new("Paused").show(egui_context.ctx_mut(), |_ui| {});
}

fn main_menu(mut egui_context: ResMut<EguiContext>) {
    egui::Window::new("Main Menu").show(egui_context.ctx_mut(), |_ui| {});
}
