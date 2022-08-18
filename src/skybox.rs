use bevy::{
    asset::LoadState,
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};

use crate::cubemap_material::CubemapMaterial;

#[derive(Clone)]
pub struct SkyboxPlugin {
    cubemap: String,
}

impl SkyboxPlugin {
    pub fn new(cubemap: impl Into<String>) -> Self {
        Self {
            cubemap: cubemap.into(),
        }
    }
}

impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.clone())
            .add_plugin(MaterialPlugin::<CubemapMaterial>::default())
            .add_startup_system(setup)
            .add_system(spawn_skybox);
    }
}

struct Skybox {
    handle: Handle<Image>,
    is_spawned: bool,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, plugin: Res<SkyboxPlugin>) {
    let handle = asset_server.load(&plugin.cubemap);

    commands.insert_resource(Skybox {
        handle,
        is_spawned: false,
    });
}

fn spawn_skybox(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cubemap_materials: ResMut<Assets<CubemapMaterial>>,
    mut skybox: ResMut<Skybox>,
) {
    if skybox.is_spawned {
        return;
    } else if asset_server.get_load_state(skybox.handle.clone_weak()) != LoadState::Loaded {
        info!("Waiting for skybox to load...");
        return;
    }

    info!("Spawning skybox");

    let mut image = images.get_mut(&skybox.handle).unwrap();
    if image.texture_descriptor.array_layer_count() == 1 {
        image.reinterpret_stacked_2d_as_array(
            image.texture_descriptor.size.height / image.texture_descriptor.size.width,
        );
        image.texture_view_descriptor = Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..Default::default()
        });
    }

    commands.spawn_bundle(MaterialMeshBundle::<CubemapMaterial> {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 10000.0 })),
        material: cubemap_materials.add(CubemapMaterial {
            base_color_texture: Some(skybox.handle.clone_weak()),
        }),
        ..Default::default()
    });

    skybox.is_spawned = true;
}
