//! In-game HUD

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::{
    metadata::{FighterMeta, GameMeta},
    ui::widgets::{bordered_frame::BorderedFrame, progress_bar::ProgressBar, EguiUIExt},
    Player, Stats,
};

pub fn render_hud(
    mut egui_context: ResMut<EguiContext>,
    players: Query<(&Stats, &Handle<FighterMeta>), With<Player>>,
    game: Res<GameMeta>,
    fighter_assets: Res<Assets<FighterMeta>>,
) {
    let ui_theme = &game.ui_theme;

    // Helper struct for holding player hud info
    struct PlayerInfo {
        name: String,
        life: f32,
        portrait_texture_id: egui::TextureId,
        portrait_size: egui::Vec2,
    }

    // Collect player info
    let mut player_infos = Vec::new();
    for (stats, fighter_handle) in players.iter() {
        if let Some(fighter) = fighter_assets.get(fighter_handle) {
            let portrait_size = fighter.hud.portrait.image_size;
            player_infos.push(PlayerInfo {
                name: fighter.name.clone(),
                life: stats.health as f32 / fighter.stats.health as f32,
                portrait_texture_id: egui_context
                    .add_image(fighter.hud.portrait.image_handle.clone_weak()),
                portrait_size: egui::Vec2::new(portrait_size.x, portrait_size.y),
            });
        }
    }

    let border = ui_theme.hud.portrait_frame.border_size;
    let scale = ui_theme.hud.portrait_frame.scale;
    let portrait_frame_padding = egui::style::Margin {
        left: border.left * scale,
        right: border.right * scale,
        top: border.top * scale,
        bottom: border.bottom * scale,
    };

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(egui_context.ctx_mut(), |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                for player in player_infos {
                    ui.add_space(20.0);

                    ui.vertical(|ui| {
                        ui.allocate_ui(egui::Vec2::new(ui_theme.hud.player_hud_width, 50.), |ui| {
                            ui.themed_label(&ui_theme.hud.font, &player.name);

                            ui.horizontal(|ui| {
                                BorderedFrame::new(&ui_theme.hud.portrait_frame)
                                    .padding(portrait_frame_padding)
                                    .show(ui, |ui| {
                                        ui.image(player.portrait_texture_id, player.portrait_size);
                                    });

                                ui.vertical(|ui| {
                                    ui.add_space(5.0);
                                    ProgressBar::new(&ui_theme.hud.lifebar, player.life)
                                        .min_width(ui.available_width())
                                        .show(ui);
                                });
                            });
                        });
                    });
                }
            });
        });
}
