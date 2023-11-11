#import bevy_pbr::mesh_functions as mesh_functions
#import bevy_pbr::skinning
#import bevy_pbr::morph
#import bevy_pbr::mesh_bindings       mesh
#import bevy_pbr::mesh_vertex_output  MeshVertexOutput
#import bevy_pbr::mesh_view_bindings as view_bindings
#import bevy_pbr::mesh_view_types

struct Vertex {
#ifdef VERTEX_POSITIONS
    @location(0) position: vec3<f32>,
#endif
#ifdef VERTEX_NORMALS
    @location(1) normal: vec3<f32>,
#endif
#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(3) tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
#endif
#ifdef SKINNED
    @location(5) joint_indices: vec4<u32>,
    @location(6) joint_weights: vec4<f32>,
#endif
#ifdef MORPH_TARGETS
    @builtin(vertex_index) index: u32,
#endif
};

@group(1) @binding(0)
var<uniform> settings: OceanSettings;
@group(1) @binding(1)
var displacement_textures: texture_2d_array<f32>;
@group(1) @binding(2)
var displacement_sampler: sampler;
@group(1) @binding(3)
var gradient_textures: texture_2d_array<f32>;
@group(1) @binding(4)
var gradient_sampler: sampler;


struct OceanSettings {
    displacement_depth_attenuation: f32,

    tile_layers: vec4<f32>,
    contribute_layers: vec4<f32>,

#ifdef SIXTEEN_BYTE_ALIGNMENT
    _webgl_padding: vec3<f32>,
#endif
}


@vertex
fn vertex(vertex: Vertex) -> MeshVertexOutput {
    let uv = vertex.uv;
    let dimensions = vec2<f32>(textureDimensions(displacement_textures)) - 0.5;

    let uv1 = fract(uv * settings.tile_layers.x);
    let uv2 = fract((uv - 0.5) * settings.tile_layers.y);
    let uv3 = fract((uv - 1.125) * settings.tile_layers.z);
    let uv4 = fract((uv - 1.25) * settings.tile_layers.w);

    let displacement_1 = textureLoad(displacement_textures, vec2<i32>(uv1 * dimensions), 0, 0) * settings.contribute_layers.x; 
    let displacement_2 = textureLoad(displacement_textures, vec2<i32>(uv2 * dimensions), 1, 0) * settings.contribute_layers.y; 
    let displacement_3 = textureLoad(displacement_textures, vec2<i32>(uv3 * dimensions), 2, 0) * settings.contribute_layers.z; 
    let displacement_4 = textureLoad(displacement_textures, vec2<i32>(uv4 * dimensions), 3, 0) * settings.contribute_layers.w; 
    let displacement = displacement_1.xyz + displacement_2.xyz + displacement_3.xyz + displacement_4.xyz;

    #ifdef SKINNED
        var model = bevy_pbr::skinning::skin_model(vertex.joint_indices, vertex.joint_weights);
    #else
        var model = mesh.model;
    #endif

    let position = vertex.position + displacement.xyz;
    // let position = vertex.position + (vec4(displacement, 1.0) * mesh.inverse_transpose_model).xyz;

    var out: MeshVertexOutput;

    #ifdef VERTEX_NORMALS
    #ifdef SKINNED
        out.world_normal = bevy_pbr::skinning::skin_normals(model, vertex.normal);
    #else
        out.world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal);
    #endif
    #endif

    #ifdef VERTEX_POSITIONS
        out.world_position = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(position, 1.0));
        out.position = mesh_functions::mesh_position_world_to_clip(out.world_position);
    #endif

    #ifdef VERTEX_UVS
        out.uv = vertex.uv;
    #endif

    #ifdef VERTEX_TANGENTS
        out.world_tangent = mesh_functions::mesh_tangent_local_to_world(model, vertex.tangent);
    #endif

    #ifdef VERTEX_COLORS
        out.color = vertex.color;
    #endif

    return out;
}


@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let directional_light = view_bindings::lights.directional_lights[0u];
    let to_light = directional_light.direction_to_light;

    let uv1 = fract(in.uv * settings.tile_layers.x);
    let uv2 = fract((in.uv - 0.5) * settings.tile_layers.y);
    let uv3 = fract((in.uv - 1.125) * settings.tile_layers.z);
    let uv4 = fract((in.uv - 1.25) * settings.tile_layers.w);
    let dimensions = vec2<f32>(textureDimensions(gradient_textures)) - 0.5;
    let gradient_1 = textureLoad(gradient_textures, vec2<i32>(uv1 * dimensions), 0, 0) * settings.contribute_layers.x; 
    let gradient_2 = textureLoad(gradient_textures, vec2<i32>(uv2 * dimensions), 1, 0) * settings.contribute_layers.y; 
    let gradient_3 = textureLoad(gradient_textures, vec2<i32>(uv3 * dimensions), 2, 0) * settings.contribute_layers.z; 
    let gradient_4 = textureLoad(gradient_textures, vec2<i32>(uv4 * dimensions), 3, 0) * settings.contribute_layers.w; 
    let gradient = gradient_1.xyz + gradient_2.xyz + gradient_3.xyz + gradient_4.xyz;

    let macro_normal = vec3(0.0, 1.0, 0.0);
    var normal = vec3(-gradient.x, 1.0, -gradient.y);

    let diffuse = saturate(dot(normal, to_light));

    return vec4(vec3(diffuse), 1.0);
}