#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::pbr_types
#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::pbr_functions

struct MyMaterial {
    color: vec4<f32>,
    time: f32,
    opacity: f32,
};

@group(1) @binding(0)
var<uniform> material: MyMaterial;

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
};

@fragment
fn fragment(
    in: FragmentInput
) -> @location(0) vec4<f32> {
    var pbr_input: PbrInput = pbr_input_new();
    pbr_input.material.base_color = material.color;
    
#ifdef VERTEX_COLORS
    pbrpbr_input.material.base_color = pbrpbr_input.material.base_color * in.color;
#endif

    pbr_input.frag_coord = in.frag_coord;
    pbr_input.world_position = in.world_position;
    pbr_input.world_normal = in.world_normal;

    pbr_input.is_orthographic = view.projection[3].w == 1.0;

    pbr_input.N = prepare_normal(
        pbr_input.material.flags,
        in.world_normal,
#ifdef VERTEX_TANGENTS
#ifdef STANDARDMATERIAL_NORMAL_MAP
        in.world_tangent,
#endif
#endif
        in.uv,
        in.is_front,
    );
    pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);

    var output: vec4<f32> =  tone_mapping(pbr(pbr_input));
    output.w = material.opacity;

    return output;
}