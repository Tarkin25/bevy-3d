use bevy::{pbr::NotShadowCaster, prelude::*};

pub struct SkyboxPlugin;

impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(Skybox::spawn)
            .add_system(Skybox::sync_with_camera);
    }
}

#[derive(Component, Debug)]
pub struct Skybox;

impl Skybox {
    fn spawn(
        mut commands: Commands,
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::default())),
                material: materials.add(StandardMaterial {
                    base_color: Color::BLUE,
                    unlit: true,
                    cull_mode: None,
                    ..Default::default()
                }),
                transform: Transform::from_scale(Vec3::splat(2000.0)),
                ..Default::default()
            },
            NotShadowCaster,
            Skybox,
        ));
    }

    fn sync_with_camera(
        mut skybox_query: Query<&mut Transform, (With<Skybox>, Without<Camera>)>,
        camera_query: Query<&Transform, (With<Camera>, Without<Skybox>)>,
    ) {
        let mut skybox = skybox_query.single_mut();
        let camera = camera_query.single();

        skybox.translation = camera.translation;
    }
}
