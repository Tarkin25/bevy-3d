#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::pbr_bindings
#import bevy_pbr::mesh_bindings

#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::pbr_functions

struct TextureAtlasMaterial {
    size: vec2<f32>,
    resolution: f32,
    gap: f32,
};

@group(1) @binding(0)
var<uniform> texture_atlas_material: TextureAtlasMaterial;

@group(1) @binding(1)
var texture_atlas: texture_2d<f32>;

@group(1) @binding(2)
var color_sampler: sampler;

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var pbr_input: PbrInput = pbr_input_new();
    var gap = vec2<f32>(0.0, 0.0);
    if in.uv.x > 0.0 {
        gap.x = texture_atlas_material.gap;
    }
    if in.uv.y > 0.0 {
        gap.y = texture_atlas_material.gap;
    }
    var uv = vec2<f32>(
        in.uv.x * (texture_atlas_material.resolution + gap.x) / texture_atlas_material.size.x,
        in.uv.y * (texture_atlas_material.resolution + gap.y) / texture_atlas_material.size.y,
    );
    //var uv = vec2<f32>(in.uv.x / texture_atlas_material.size.x, in.uv.y / texture_atlas_material.size.y);

    pbr_input.material.base_color = pbr_input.material.base_color * textureSample(texture_atlas, color_sampler, uv);
    pbr_input.frag_coord = in.frag_coord;
    pbr_input.world_position = in.world_position;
    pbr_input.N = prepare_normal(
        pbr_input.material.flags,
        in.world_normal,
#ifdef VERTEX_TANGENTSin
#ifdef STANDARDMATERIAL_NORMAL_MAP
        in.world_tangent,
#endif
#endif
        in.uv,
        in.is_front,
    );

    pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);

    return tone_mapping(pbr(pbr_input));
}