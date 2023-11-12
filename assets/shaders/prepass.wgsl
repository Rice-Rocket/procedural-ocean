#import ocean::main displacement_textures
#import ocean::main displacement_sampler
#import ocean::main settings
#import bevy_pbr::prepass_bindings
#import bevy_pbr::mesh_functions
#import bevy_pbr::skinning
#import bevy_pbr::morph
#import bevy_pbr::mesh_bindings mesh


struct Vertex {
    @location(0) position: vec3<f32>,

#ifdef VERTEX_UVS
    @location(1) uv: vec2<f32>,
#endif // VERTEX_UVS

#ifdef NORMAL_PREPASS
    @location(2) normal: vec3<f32>,
#ifdef VERTEX_TANGENTS
    @location(3) tangent: vec4<f32>,
#endif // VERTEX_TANGENTS
#endif // NORMAL_PREPASS

#ifdef SKINNED
    @location(4) joint_indices: vec4<u32>,
    @location(5) joint_weights: vec4<f32>,
#endif // SKINNED

#ifdef MORPH_TARGETS
    @builtin(vertex_index) index: u32,
#endif // MORPH_TARGETS
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,

#ifdef VERTEX_UVS
    @location(0) uv: vec2<f32>,
#endif // VERTEX_UVS

#ifdef NORMAL_PREPASS
    @location(1) world_normal: vec3<f32>,
#ifdef VERTEX_TANGENTS
    @location(2) world_tangent: vec4<f32>,
#endif // VERTEX_TANGENTS
#endif // NORMAL_PREPASS

#ifdef MOTION_VECTOR_PREPASS
    @location(3) world_position: vec4<f32>,
    @location(4) previous_world_position: vec4<f32>,
#endif // MOTION_VECTOR_PREPASS

#ifdef DEPTH_CLAMP_ORTHO
    @location(5) clip_position_unclamped: vec4<f32>,
#endif // DEPTH_CLAMP_ORTHO
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

#ifdef SKINNED
    var model = bevy_pbr::skinning::skin_model(vertex.joint_indices, vertex.joint_weights);
#else // SKINNED
    var model = mesh.model;
#endif // SKINNED

    let uv = vertex.uv;

    let uv1 = fract(uv * settings.tile_layers.x);
    let uv2 = fract((uv - 0.5) * settings.tile_layers.y);
    let uv3 = fract((uv - 1.125) * settings.tile_layers.z);
    let uv4 = fract((uv - 1.25) * settings.tile_layers.w);

    let displacement_1 = textureSampleLevel(displacement_textures, displacement_sampler, uv1, 0, 0.0) * settings.contribute_layers.x; 
    let displacement_2 = textureSampleLevel(displacement_textures, displacement_sampler, uv2, 1, 0.0) * settings.contribute_layers.y; 
    let displacement_3 = textureSampleLevel(displacement_textures, displacement_sampler, uv3, 2, 0.0) * settings.contribute_layers.z; 
    let displacement_4 = textureSampleLevel(displacement_textures, displacement_sampler, uv4, 3, 0.0) * settings.contribute_layers.w; 
    let displacement = displacement_1.xyz + displacement_2.xyz + displacement_3.xyz + displacement_4.xyz;

    let position = vertex.position + displacement;

    out.clip_position = bevy_pbr::mesh_functions::mesh_position_local_to_clip(model, vec4(position, 1.0));
    out.clip_position_unclamped = out.clip_position;
    out.clip_position.z = min(out.clip_position.z, 1.0);

    return out;
}

struct FragmentInput {
#ifdef VERTEX_UVS
    @location(0) uv: vec2<f32>,
#endif // VERTEX_UVS

#ifdef NORMAL_PREPASS
    @location(1) world_normal: vec3<f32>,
#endif // NORMAL_PREPASS

#ifdef MOTION_VECTOR_PREPASS
    @location(3) world_position: vec4<f32>,
    @location(4) previous_world_position: vec4<f32>,
#endif // MOTION_VECTOR_PREPASS

#ifdef DEPTH_CLAMP_ORTHO
    @location(5) clip_position_unclamped: vec4<f32>,
#endif // DEPTH_CLAMP_ORTHO
}

struct FragmentOutput {
#ifdef NORMAL_PREPASS
    @location(0) normal: vec4<f32>,
#endif // NORMAL_PREPASS

#ifdef MOTION_VECTOR_PREPASS
    @location(1) motion_vector: vec2<f32>,
#endif // MOTION_VECTOR_PREPASS

#ifdef DEPTH_CLAMP_ORTHO
    @builtin(frag_depth) frag_depth: f32,
#endif // DEPTH_CLAMP_ORTHO
}

@fragment
fn fragment(in: FragmentInput) -> FragmentOutput {
    var out: FragmentOutput;

    out.frag_depth = in.clip_position_unclamped.z;
    
    return out;
}