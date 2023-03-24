use bevy::{app::AppExit, prelude::*, window::CursorGrabMode};
use bevy_egui::{
    egui::{self, Align2},
    EguiContexts,
};

use crate::{
    game::chunk::{
        generator::ChunkGenerator,
        grid::{ChunkGrid, GridCoordinates},
        DespawnChunk,
    },
    settings::Settings,
    AppState,
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(configure_egui)
            .add_system(free_cursor.in_schedule(OnEnter(AppState::Menu)))
            .add_system(show_menu.in_set(OnUpdate(AppState::Menu)))
            .add_systems(
                (capture_cursor, apply_noise_settings).in_schedule(OnEnter(AppState::InGame)),
            );
    }
}

fn free_cursor(mut windows: Query<&mut Window>) {
    for mut window in &mut windows {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
        let cursor_position = Vec2::new(window.width() / 2.0, window.height() / 2.0);
        window.set_cursor_position(Some(cursor_position));
    }
}

fn capture_cursor(mut windows: Query<&mut Window>) {
    for mut window in &mut windows {
        window.cursor.grab_mode = CursorGrabMode::Confined;
        window.cursor.visible = false;
    }
}

fn configure_egui() {}

fn show_menu(
    mut context: EguiContexts,
    mut state: ResMut<NextState<AppState>>,
    mut exit: EventWriter<AppExit>,
) {
    egui::Window::new("menu")
        .title_bar(false)
        .auto_sized()
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .show(context.ctx_mut(), move |ui| {
            if ui
                .add_sized([ui.available_width(), 0.0], egui::Button::new("Resume"))
                .clicked()
            {
                state.set(AppState::InGame);
            } else if ui
                .add_sized([ui.available_width(), 0.0], egui::Button::new("Exit"))
                .clicked()
            {
                exit.send(AppExit);
            }
        });
}

fn apply_noise_settings(
    mut commands: Commands,
    mut settings: ResMut<Settings>,
    generator: Res<ChunkGenerator>,
    chunks: Query<(Entity, &GridCoordinates)>,
    grid: Res<ChunkGrid>,
) {
    if settings.detect_changes() {
        generator.set_scale(settings.noise.scale);

        for (entity, coordinates) in chunks.iter() {
            grid.remove(coordinates);
            commands.entity(entity).insert(DespawnChunk);
        }
    }
}
