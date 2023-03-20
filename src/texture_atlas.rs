use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

pub struct TextureAtlasPlugin;

impl Plugin for TextureAtlasPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<TextureAtlasMaterial>::default());
    }
}

#[derive(AsBindGroup, Clone, Debug, TypeUuid)]
#[uuid = "c51252b4-4fd6-4c02-9abb-2bf93d8e3bb3"]
pub struct TextureAtlasMaterial {
    #[uniform(0)]
    pub size: Vec2,
    #[uniform(0)]
    pub resolution: f32,
    #[uniform(0)]
    pub gap: f32,
    #[texture(1)]
    #[sampler(2)]
    pub texture_handle: Handle<Image>,
}

impl Material for TextureAtlasMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/texture_atlas_material.wgsl".into()
    }
}
