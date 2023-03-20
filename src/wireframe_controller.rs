use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
};

pub struct WireframeControllerPlugin;

impl Plugin for WireframeControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(WireframePlugin).add_system(toggle_wireframe);
    }
}

fn toggle_wireframe(mut config: ResMut<WireframeConfig>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::W) {
        config.global = !config.global;
    }
}
