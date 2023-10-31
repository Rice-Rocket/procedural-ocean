#import bevy_pbr::mesh_vertex_output MeshVertexOutput
#import bevy_pbr::mesh_view_bindings as view_bindings
#import bevy_pbr::clustered_forward as clustering
#import bevy_pbr::mesh_view_types as mesh_view_types
#import bevy_pbr::shadows as shadows
#import bevy_pbr::pbr_functions as pbr_functions
#import bevy_pbr::pbr_types as pbr_types
#import bevy_pbr::lighting as pbr_lighting
#import bevy_pbr::mesh_bindings mesh
#import bevy_pbr::mesh_types MESH_FLAGS_SHADOW_RECEIVER_BIT


@group(0) @binding(0)
var<uniform> settings: OceanSettings;


struct OceanSettings {
    wave_height: f32,

#ifdef SIXTEEN_BYTE_ALIGNMENT
    _webgl_padding: vec3<f32>,
#endif
}


@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    return vec4(1.0);
}