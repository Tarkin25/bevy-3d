use std::time::Duration;

use bevy::prelude::*;
use bevy_atmosphere::{prelude::Nishita, system_param::AtmosphereMut};
use bevy_inspector_egui::{prelude::*, quick::ResourceInspectorPlugin};

use crate::AppState;

pub struct DaylightCyclePlugin;

impl Plugin for DaylightCyclePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CycleTimer(Timer::new(
            Duration::from_millis(100),
            TimerMode::Repeating,
        )))
        .insert_resource(DaylightCycleSettings { speed: 1.0 })
        .add_plugins(ResourceInspectorPlugin::<DaylightCycleSettings>::default())
        .add_systems(Update, Sun::cycle)
        .add_systems(OnExit(AppState::InGame), pause)
        .add_systems(OnEnter(AppState::InGame), unpause);
    }
}

#[derive(Component)]
pub struct Sun;

impl Sun {
    fn cycle(
        mut atmosphere: AtmosphereMut<Nishita>,
        mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
        mut timer: ResMut<CycleTimer>,
        time: Res<Time<Virtual>>,
        settings: Res<DaylightCycleSettings>,
    ) {
        timer.0.tick(time.delta());

        if settings.speed != 0.0 && timer.0.finished() {
            let delta = time.elapsed_seconds_wrapped() * settings.speed / 100.0;
            atmosphere.sun_position = Vec3::new(0.0, delta.sin(), delta.cos());

            if let Some((mut transform, mut light)) = query.get_single_mut().ok() {
                transform.rotation = Quat::from_rotation_x(-delta.sin().atan2(delta.cos()));
                light.illuminance = delta.sin().max(0.0).powf(2.0) * 100_000.0;
            }
        }
    }
}

fn pause(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

fn unpause(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
}

#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct DaylightCycleSettings {
    #[inspector(min = 0.0)]
    pub speed: f32,
}

#[derive(Resource)]
struct CycleTimer(Timer);
