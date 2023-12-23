use bevy::{prelude::*, render::view::ColorGrading};
use bevy_inspector_egui::{prelude::*, quick::ResourceInspectorPlugin, InspectorOptions};

pub struct ColorGradingPlugin;

impl Plugin for ColorGradingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ColorGradingSettings>()
            .add_plugins(ResourceInspectorPlugin::<ColorGradingSettings>::default())
            .add_systems(Update, ColorGradingSettings::apply);
    }
}

#[derive(Debug, Resource, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
struct ColorGradingSettings {
    exposure: f32,
    gamma: f32,
    pre_saturation: f32,
    post_saturation: f32,
}

impl ColorGradingSettings {
    fn apply(settings: Res<ColorGradingSettings>, mut query: Query<&mut ColorGrading>) {
        for mut color_grading in &mut query {
            color_grading.exposure = settings.exposure;
            color_grading.gamma = settings.gamma;
            color_grading.pre_saturation = settings.pre_saturation;
            color_grading.post_saturation = settings.post_saturation;
        }
    }
}

impl Default for ColorGradingSettings {
    fn default() -> Self {
        let ColorGrading {
            exposure,
            gamma,
            pre_saturation,
            post_saturation,
        } = ColorGrading::default();

        Self {
            exposure,
            gamma,
            pre_saturation,
            post_saturation,
        }
    }
}
