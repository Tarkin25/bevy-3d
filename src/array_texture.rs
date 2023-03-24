use bevy::{
    asset::LoadState,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{MeshVertexAttribute, MeshVertexBufferLayout},
        render_asset::RenderAssets,
        render_resource::{
            encase::UniformBuffer, AsBindGroup, AsBindGroupError, BindGroupDescriptor,
            BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
            BindingType, BufferBindingType, BufferInitDescriptor, BufferUsages,
            OwnedBindingResource, PreparedBindGroup, RenderPipelineDescriptor, SamplerBindingType,
            ShaderRef, ShaderStages, ShaderType, SpecializedMeshPipelineError, TextureSampleType,
            TextureViewDimension, VertexFormat,
        },
        renderer::RenderDevice,
        texture::FallbackImage,
    },
};

use crate::VoxelConfig;

pub struct ArrayTexturePlugin;

impl Plugin for ArrayTexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<ArrayTextureMaterial>::default())
            .add_system(ArrayTextureMaterial::initialize_length)
            .add_system(ArrayTextureMaterial::reinterpret_image);
    }
}

#[derive(Clone, Debug, TypeUuid)]
#[uuid = "315231f6-e552-439a-9796-f0a819e9a645"]
pub struct ArrayTextureMaterial {
    length: Length,
    texture: Handle<Image>,
    image_reinterpreted: bool,
}

#[derive(Clone, Copy, Debug)]
enum Length {
    Initialized(u32),
    Uninitialized(f32),
}

pub const ATTRIBUTE_TEXTURE_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("ATTRIBUTE_TEXTURE_INDEX", 100, VertexFormat::Uint32);

impl ArrayTextureMaterial {
    pub fn with_length(texture: Handle<Image>, length: u32) -> Self {
        Self {
            texture,
            length: Length::Initialized(length),
            image_reinterpreted: false,
        }
    }

    pub fn with_resolution(texture: Handle<Image>, resolution: f32) -> Self {
        Self {
            texture,
            length: Length::Uninitialized(resolution),
            image_reinterpreted: false,
        }
    }

    fn initialize_length(
        voxel_config: Res<VoxelConfig>,
        mut materials: ResMut<Assets<ArrayTextureMaterial>>,
        images: Res<Assets<Image>>,
        asset_server: Res<AssetServer>,
    ) {
        let material_handle = &voxel_config.material;
        let material = materials.get_mut(material_handle).unwrap();

        if let Length::Uninitialized(resolution) = material.length {
            if let LoadState::Loaded = asset_server.get_load_state(&material.texture) {
                let image = images.get(&material.texture).unwrap();
                let length = image.size().y / resolution;
                assert_eq!(length % 1.0, 0.0);
                material.length = Length::Initialized(length as u32);
            }
        }
    }

    fn reinterpret_image(
        voxel_config: Res<VoxelConfig>,
        mut materials: ResMut<Assets<ArrayTextureMaterial>>,
        mut images: ResMut<Assets<Image>>,
        asset_server: Res<AssetServer>,
    ) {
        let material_handle = &voxel_config.material;
        let material = materials.get_mut(material_handle).unwrap();

        if let Length::Initialized(length) = material.length {
            if let LoadState::Loaded = asset_server.get_load_state(&material.texture) {
                if !material.image_reinterpreted {
                    let image = images.get_mut(&material.texture).unwrap();
                    image.reinterpret_stacked_2d_as_array(length);
                    material.image_reinterpreted = true;
                }
            }
        }
    }
}

impl Material for ArrayTextureMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/array_texture.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/array_texture.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            // Mesh::ATTRIBUTE_TANGENT.at_shader_location(3),
            // Mesh::ATTRIBUTE_COLOR.at_shader_location(4),
            // Mesh::ATTRIBUTE_JOINT_INDEX.at_shader_location(5),
            // Mesh::ATTRIBUTE_JOINT_WEIGHT.at_shader_location(6),
            ATTRIBUTE_TEXTURE_INDEX.at_shader_location(7),
        ])?;

        let shader_defs = ["VERTEX_POSITIONS", "VERTEX_NORMALS", "VERTEX_UVS"].map(|s| s.into());

        descriptor.vertex.shader_defs.extend(shader_defs.clone());
        if let Some(fragment) = descriptor.fragment.as_mut() {
            fragment.shader_defs.extend(shader_defs);
        }

        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

impl AsBindGroup for ArrayTextureMaterial {
    type Data = ();
    fn as_bind_group(
        &self,
        layout: &BindGroupLayout,
        render_device: &RenderDevice,
        images: &RenderAssets<Image>,
        fallback_image: &FallbackImage,
    ) -> Result<PreparedBindGroup<()>, AsBindGroupError> {
        match self.length {
            Length::Initialized(length) if self.image_reinterpreted => {
                self.create_bind_group(layout, render_device, images, fallback_image, length)
            }
            _ => Err(AsBindGroupError::RetryNextUpdate),
        }
    }
    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 1u32,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2Array,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2u32,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 0u32,
                    visibility: ShaderStages::all(),
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(<u32 as ShaderType>::min_size()),
                    },
                    count: None,
                },
            ],
            label: None,
        })
    }
}

impl ArrayTextureMaterial {
    fn create_bind_group(
        &self,
        layout: &BindGroupLayout,
        render_device: &RenderDevice,
        images: &RenderAssets<Image>,
        fallback_image: &FallbackImage,
        length: u32,
    ) -> Result<PreparedBindGroup<()>, AsBindGroupError> {
        let bindings = vec![
            OwnedBindingResource::TextureView({
                let handle: Option<&bevy::asset::Handle<Image>> = (&self.texture).into();
                if let Some(handle) = handle {
                    images
                        .get(handle)
                        .ok_or_else(|| AsBindGroupError::RetryNextUpdate)?
                        .texture_view
                        .clone()
                } else {
                    fallback_image.texture_view.clone()
                }
            }),
            OwnedBindingResource::Sampler({
                let handle: Option<&bevy::asset::Handle<Image>> = (&self.texture).into();
                if let Some(handle) = handle {
                    images
                        .get(handle)
                        .ok_or_else(|| AsBindGroupError::RetryNextUpdate)?
                        .sampler
                        .clone()
                } else {
                    fallback_image.sampler.clone()
                }
            }),
            {
                let mut buffer = UniformBuffer::new(Vec::new());
                buffer.write(&length).unwrap();
                OwnedBindingResource::Buffer(render_device.create_buffer_with_data(
                    &BufferInitDescriptor {
                        label: None,
                        usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
                        contents: buffer.as_ref(),
                    },
                ))
            },
        ];
        let bind_group = {
            let descriptor = BindGroupDescriptor {
                entries: &[
                    BindGroupEntry {
                        binding: 1u32,
                        resource: bindings[0usize].get_binding(),
                    },
                    BindGroupEntry {
                        binding: 2u32,
                        resource: bindings[1usize].get_binding(),
                    },
                    BindGroupEntry {
                        binding: 0u32,
                        resource: bindings[2usize].get_binding(),
                    },
                ],
                label: None,
                layout: &layout,
            };
            render_device.create_bind_group(&descriptor)
        };
        Ok(PreparedBindGroup {
            bindings,
            bind_group,
            data: (),
        })
    }
}
