use bevy::{
    asset::LoadState,
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

pub struct TextureAtlasPlugin;

impl Plugin for TextureAtlasPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<TextureAtlasMaterial>::default())
            .add_system(TextureAtlasMaterial::init_size_and_resolution);
    }
}

#[derive(AsBindGroup, Clone, Debug, TypeUuid)]
#[uuid = "c51252b4-4fd6-4c02-9abb-2bf93d8e3bb3"]
pub struct TextureAtlasMaterial {
    #[uniform(0)]
    size: Vec2,
    #[uniform(0)]
    resolution: f32,
    #[uniform(0)]
    gap: f32,
    #[texture(1)]
    #[sampler(2)]
    texture_handle: Handle<Image>,
    rows: usize,
    columns: usize,
}

impl TextureAtlasMaterial {
    pub fn new(texture_handle: Handle<Image>, rows: usize, columns: usize, gap: f32) -> Self {
        Self {
            size: Vec2::ZERO,
            resolution: 0.0,
            gap,
            texture_handle,
            rows,
            columns,
        }
    }

    fn init_size_and_resolution(
        mut materials: ResMut<Assets<TextureAtlasMaterial>>,
        query: Query<&Handle<TextureAtlasMaterial>>,
        asset_server: Res<AssetServer>,
        images: Res<Assets<Image>>,
    ) {
        for material_handle in &query {
            let mut material = materials.get_mut(material_handle).unwrap();

            if matches!(
                asset_server.get_load_state(&material.texture_handle),
                LoadState::Loaded
            ) {
                let image = images.get(&material.texture_handle).unwrap();
                material.size = image.size();
                // (resolution + gap) * columns = size
                // resolution = size / columns - gap
                material.resolution = material.size.x / material.columns as f32 - material.gap;
                assert_eq!(
                    material.resolution,
                    material.size.y / material.rows as f32 - material.gap
                );
            }
        }
    }
}

impl Material for TextureAtlasMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/texture_atlas_material.wgsl".into()
    }
}
