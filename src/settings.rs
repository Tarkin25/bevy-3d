use bevy::prelude::*;

use crate::mesh_builder::MeshBuilderSettings;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Settings::default())
        .add_startup_system(setup)
        .add_system(update_settings_display)
        .add_system(update_render_distance);
    }
}

fn setup(mut commands: Commands, settings: Res<Settings>, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(
        TextBundle::from_sections([
            TextSection::new("Render Distance: ", TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                ..Default::default()
            }),
            TextSection::new(settings.render_distance.to_string(), TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                ..Default::default()
            })
        ])
        .with_style(Style {
            align_self: AlignSelf::FlexEnd,
            ..Default::default()
        })
    ).insert(SettingsDisplay);
}

fn update_settings_display(mut text: Query<&mut Text, With<SettingsDisplay>>, settings: Res<Settings>) {
    if settings.is_changed() {
        let mut text = text.single_mut();
        text.sections[1].value = settings.render_distance.to_string();
    }
}

fn update_render_distance(input: Res<Input<KeyCode>>, mut settings: ResMut<Settings>) {
    if input.just_pressed(KeyCode::W) {
        settings.render_distance += 1;
    }
    if input.just_pressed(KeyCode::S) {
        settings.render_distance -= 1;
    }
}

#[derive(Component)]
struct SettingsDisplay;

#[derive(Clone, Copy)]
pub struct Settings {
    pub render_distance: isize,
    pub mesh_builder: MeshBuilderSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            render_distance: 16,
            mesh_builder: Default::default()
        }
    }
}