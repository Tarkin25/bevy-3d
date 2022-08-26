use std::sync::{Arc, RwLock};

use bevy::{prelude::*, app::AppExit};
use bevy_egui::{egui::{self, Align2}, EguiContext};

use crate::{AppState, settings::Settings, game::chunk::{generator::ChunkGenerator, grid::{GridCoordinates, ChunkGrid}, DespawnChunk}};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_startup_system(configure_egui)
        .add_system_set(
            SystemSet::on_enter(AppState::Menu)
                .with_system(free_cursor),
        )
        .add_system_set(SystemSet::on_update(AppState::Menu).with_system(show_menu))
        .add_system_set(
            SystemSet::on_exit(AppState::Menu)
                .with_system(capture_cursor)
                .with_system(apply_noise_settings),
        );
    }
}

fn free_cursor(mut windows: ResMut<Windows>) {
    for window in windows.iter_mut() {
        window.set_cursor_visibility(true);
        window.set_cursor_lock_mode(false);
        window.set_cursor_position(Vec2::new(window.width() / 2.0, window.height() / 2.0));
    }
}

fn capture_cursor(mut windows: ResMut<Windows>) {
    for window in windows.iter_mut() {
        window.set_cursor_visibility(false);
        window.set_cursor_lock_mode(true);
    }
}

fn configure_egui() {
    
}

fn show_menu(mut context: ResMut<EguiContext>, mut settings: ResMut<Settings>, mut state: ResMut<State<AppState>>, mut exit: EventWriter<AppExit>) {
    egui::Window::new("menu")
    .title_bar(false)
    .auto_sized()
    .collapsible(false)
    .resizable(false)
    .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
    .show(context.ctx_mut(), move |ui| {
        ui.add(egui::Slider::new(&mut settings.render_distance, 0..=30).show_value(true).text("Render Distance"));
        ui.add(egui::DragValue::new(&mut settings.noise.octaves).prefix("Octaves: "));
        ui.add(egui::DragValue::new(&mut settings.noise.amplitude).prefix("Amplitude: "));

        if ui.button("Resume").clicked() {
            state.set(AppState::InGame).unwrap();
        } else if ui.button("Exit").clicked() {
            exit.send(AppExit);
        }
    });

    let font = egui::FontId::proportional(32.0);
    context.ctx_mut().fonts().row_height(&font);
}

fn apply_noise_settings(mut commands: Commands, mut settings: ResMut<Settings>, generator: Res<Arc<RwLock<ChunkGenerator>>>, chunks: Query<(Entity, &GridCoordinates)>, mut grid: ResMut<ChunkGrid>) {
    if settings.detect_changes() {
        generator.write().unwrap().update_settings(settings.noise);
        for (entity, coordinates) in chunks.iter() {
            grid.remove(coordinates);
            commands.entity(entity).insert(DespawnChunk);
        }
    }
}
