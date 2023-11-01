#import bevy_pbr::mesh_functions as mesh_functions
#import bevy_pbr::skinning
#import bevy_pbr::morph
#import bevy_pbr::mesh_bindings       mesh
#import bevy_pbr::mesh_vertex_output  MeshVertexOutput

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
var displacement_texture: texture_2d<f32>;
@group(1) @binding(2)
var displacement_sampler: sampler;


struct OceanSettings {
    time: f32,
    frequency: f32,
    amplitude: f32,

#ifdef SIXTEEN_BYTE_ALIGNMENT
    _webgl_padding: f32,
#endif
}


@vertex
fn vertex(vertex: Vertex) -> MeshVertexOutput {
    let uv = vertex.uv;
    let dimensions = vec2<f32>(textureDimensions(displacement_texture)) - 0.5;

    let displacements = textureLoad(displacement_texture, vec2<i32>(uv * dimensions), 0); 
    let height = displacements.x;
    
    let position = vec3(vertex.position.x, height, vertex.position.z);

    var out: MeshVertexOutput;

    #ifdef SKINNED
        var model = bevy_pbr::skinning::skin_model(vertex.joint_indices, vertex.joint_weights);
    #else
        var model = mesh.model;
    #endif

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
    return vec4(1.0);
}