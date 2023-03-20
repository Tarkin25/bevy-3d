use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::{settings::Settings, AppState};

use super::camera_controller::CameraController;

pub struct DebugInfoPlugin;

impl Plugin for DebugInfoPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(AppState::InGame).with_system(show_debug_info));
    }
}

fn show_debug_info(
    player: Query<&Transform, With<CameraController>>,
    settings: Res<Settings>,
    mut context: ResMut<EguiContext>,
) {
    let player = player.single();

    egui::Window::new("Debug Info")
        .collapsible(false)
        .resizable(false)
        .show(context.ctx_mut(), |ui| {
            ui.label(format!(
                "Position: ({:.0}, {:.0}, {:.0})",
                player.translation.x, player.translation.y, player.translation.z
            ));
            ui.label(format!(
                "Render Distance: {} chunks",
                settings.render_distance
            ));
        });
}
