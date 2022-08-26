use bevy::prelude::*;

pub struct ButtonTestPlugin;

impl Plugin for ButtonTestPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
        .add_system(button_system);
    }
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn button_system(mut interaction_query: Query<(&Interaction, &mut UiColor, &Children), (Changed<Interaction>, With<Button>)>, mut text_query: Query<&mut Text>) {
    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();

        let (text_value, color_value) = match *interaction {
            Interaction::Clicked => ("Press", PRESSED_BUTTON),
            Interaction::Hovered => ("Hover", HOVERED_BUTTON),
            Interaction::None => ("Button", NORMAL_BUTTON)
        };

        text.sections[0].value = text_value.to_string();
        *color = color_value.into();
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(ButtonBundle {
        style: Style {
            size: Size::new(Val::Px(150.0), Val::Px(65.0)),
            margin: UiRect::all(Val::Auto),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        color: NORMAL_BUTTON.into(),
        ..Default::default()
    })
    .with_children(|parent| {
        parent.spawn_bundle(TextBundle::from_section(
            "Button",
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 40.0,
                color: Color::rgb(0.9, 0.9, 0.9)
            }
        ));
    });
}