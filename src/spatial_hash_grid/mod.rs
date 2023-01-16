use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;

use crate::game::camera_controller::CameraController;

pub use self::grid::SpatialHashGrid;

mod grid;

pub struct SpatialHashGridPlugin;

impl Plugin for SpatialHashGridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpatialHashGrid::new(5))
            .add_system(track_entities)
            .add_system(mark_near_player)
            .add_system(color_near_player)
            .add_system(color_away_from_player)
            .add_startup_system(setup)
            .add_plugin(WorldInspectorPlugin::default());
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {    
    let mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let size = 30;
    let size_half = size / 2;

    for x in -size_half..size_half {
        for z in -size_half..size_half {
            commands
                .spawn_bundle(PbrBundle {
                    mesh: mesh.clone(),
                    material: materials.add(Color::GREEN.into()),
                    transform: Transform::from_xyz((x * 2) as f32, 100.0, (z * 2) as f32),
                    ..Default::default()
                })
                .insert(TrackedInGrid);
        }
    }
}

#[derive(Component)]
pub struct TrackedInGrid;

fn track_entities(
    mut grid: ResMut<SpatialHashGrid>,
    tracked: Query<(Entity, &Transform), With<TrackedInGrid>>,
) {
    tracked.for_each(|(entity, transform)| {
        grid.update(entity, transform.translation);
    });
}

#[derive(Component)]
pub struct NearPlayer;

fn mark_near_player(
    grid: Res<SpatialHashGrid>,
    player: Query<&Transform, With<CameraController>>,
    mut commands: Commands,
    tracked: Query<Entity, With<NearPlayer>>,
) {
    let transform = player.single();

    let nearby_entities = grid.get_nearby(transform.translation, 5.0);

    for entity in &nearby_entities {
        commands.entity(*entity).insert(NearPlayer);
    }

    tracked
        .iter()
        .filter(|entity| !nearby_entities.contains(&entity))
        .for_each(|entity| {
            commands.entity(entity).remove::<NearPlayer>();
        });
}

fn color_near_player(
    near_player: Query<&Handle<StandardMaterial>, Added<NearPlayer>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    near_player.for_each(|material_handle| {
        if let Some(material) = materials.get_mut(material_handle) {
            material.base_color = Color::PURPLE;
        }
    });
}

fn color_away_from_player(
    query: Query<&Handle<StandardMaterial>, (Without<NearPlayer>, With<TrackedInGrid>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    query.for_each(|material_handle| {
        if let Some(material) = materials.get_mut(material_handle) {
            material.base_color = Color::GREEN;
        }
    })
}
