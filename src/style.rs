use bevy::prelude::*;

pub struct StylePlugin;

impl Plugin for StylePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, setup_styles);
    }
}

pub fn setup_styles(mut commands: Commands, asset_server: Res<AssetServer>) {
    let styles = Styles {
        font: FontStyles {
            bold: asset_server.load("fonts/FiraSans-Bold.ttf"),
            medium: asset_server.load("fonts/FiraMono-Medium.ttf")
        },
        font_size: FontSize {
            medium: 24.0
        },
        button: ButtonStyles {
            color: ButtonColors {
                normal: Color::rgb(0.15, 0.15, 0.15),
                hovered: Color::rgb(0.25, 0.25, 0.25),
                pressed: Color::rgb(0.35, 0.75, 0.35),
            }
        }
    };

    commands.insert_resource(styles);
}

pub struct Styles {
    pub font: FontStyles,
    pub button: ButtonStyles,
    pub font_size: FontSize,
}

pub struct FontStyles {
    pub bold: Handle<Font>,
    pub medium: Handle<Font>,
}

pub struct ButtonStyles {
    pub color: ButtonColors,
}

pub struct ButtonColors {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
}

pub struct FontSize {
    pub medium: f32,
}