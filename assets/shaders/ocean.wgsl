#define_import_path ocean::main

#import bevy_pbr::mesh_functions as mesh_functions
#import bevy_pbr::skinning
#import bevy_pbr::morph
#import bevy_pbr::mesh_bindings       mesh
#import bevy_pbr::mesh_vertex_output  MeshVertexOutput
#import bevy_pbr::mesh_view_bindings as view_bindings
#import bevy_pbr::mesh_view_types
#import bevy_pbr::prepass_utils as prepass_utils

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
var<uniform> sky_settings: SkySettings;
@group(1) @binding(2)
var displacement_textures: texture_2d_array<f32>;
@group(1) @binding(3)
var displacement_sampler: sampler;
@group(1) @binding(4)
var gradient_textures: texture_2d_array<f32>;
@group(1) @binding(5)
var gradient_sampler: sampler;
@group(1) @binding(6)
var skybox_texture: texture_cube<f32>;
@group(1) @binding(7)
var skybox_sampler: sampler;


struct OceanSettings {
    base_color: vec3<f32>,
    displacement_depth_attenuation: f32,
    low_scatter: vec3<f32>,
    normal_strength: f32,
    sea_water_color: vec3<f32>,
    roughness: f32,
    sun_power: f32,
    ocean_depth: f32,
    subsurface_strength: f32,

    tile_layers: vec4<f32>,
    contribute_layers: vec4<f32>,
}

struct SkySettings {
    sun_color: vec3<f32>,
    sun_falloff: f32,
}


@vertex
fn vertex(vertex: Vertex) -> MeshVertexOutput {
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

    #ifdef SKINNED
        var model = bevy_pbr::skinning::skin_model(vertex.joint_indices, vertex.joint_weights);
    #else
        var model = mesh.model;
    #endif

    let position = vertex.position + displacement.xyz;

    var out: MeshVertexOutput;

    out.world_position = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(position, 1.0));
    out.position = mesh_functions::mesh_position_world_to_clip(out.world_position);

    out.uv = vertex.uv;

    return out;
}

const PI: f32 = 3.1415927;

fn linearize_depth(depth: f32) -> f32 {
    let far_plane = 1000.0 - 0.1;
    return mix(1.0, view_bindings::view.projection[3][2] / depth / far_plane, f32(depth > 0.0001));
    // return view_bindings::view.projection[3][2] / depth;
}

fn schlick(f0: f32, v_dot_h: f32) -> f32 {
    return f0 + (1.0 - f0) * (1.0 - v_dot_h) * (1.0 - v_dot_h) * (1.0 - v_dot_h) * (1.0 - v_dot_h) * (1.0 - v_dot_h);
}

fn henyey_greenstein(mu: f32, in_g: f32) -> f32 {
    return (1.0 - in_g * in_g) / (pow(1.0 + in_g * in_g - 2.0 * in_g * mu, 1.5) * 4.0 * PI);
}

fn d_ggx(r: f32, n_dot_h: f32, h: vec3<f32>) -> f32 {
    let a = n_dot_h * r;
    let k = r / ((1.0 - n_dot_h * n_dot_h) + a * a);
    return k * k * (1.0 / PI);
}

fn get_sky_color(dir: vec3<f32>, sun_dir: vec3<f32>) -> vec3<f32> {
    let sky = textureSample(skybox_texture, skybox_sampler, dir).xyz;
    let sun = sky_settings.sun_color * pow(saturate(dot(dir, sun_dir)), sky_settings.sun_falloff);
    return sky + sun;
}

fn get_ocean_color(p: vec3<f32>, n: vec3<f32>, sun_dir: vec3<f32>, dir: vec3<f32>, mu: f32) -> vec3<f32> {
    let l = normalize(reflect(dir, n));
    let v = -dir;
    let n_dot_v = saturate(abs(dot(n, v)) + 0.00001);
    var n_dot_l = max(0.0, dot(n, l));
    let v_dot_h = max(0.0, dot(v, normalize(v + l)));
    let fresnel = schlick(0.02, n_dot_v);
    let reflection = get_sky_color(l, sun_dir);
    var color = mix(settings.base_color, reflection, fresnel);
    let subsurface = settings.subsurface_strength * henyey_greenstein(mu, 0.5);
    color += subsurface * settings.sea_water_color * max(0.0, 1.0 + p.y - 0.6 * settings.ocean_depth);
    let h = normalize(v + sun_dir);
    n_dot_l = max(0.0, dot(n, sun_dir));
    color += settings.low_scatter * 0.4 * vec3(n_dot_l / PI * fresnel * sky_settings.sun_color * settings.sun_power * d_ggx(settings.roughness, max(0.0, dot(n, h)), h));
    return color;
}

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let dimensions = vec2<f32>(textureDimensions(gradient_textures)) - 0.5;

    let uv1 = fract(in.uv * settings.tile_layers.x);
    let uv2 = fract((in.uv - 0.5) * settings.tile_layers.y);
    let uv3 = fract((in.uv - 1.125) * settings.tile_layers.z);
    let uv4 = fract((in.uv - 1.25) * settings.tile_layers.w);
    
    let gradient_1 = textureSampleLevel(gradient_textures, gradient_sampler, uv1, 0, 0.0) * settings.contribute_layers.x; 
    let gradient_2 = textureSampleLevel(gradient_textures, gradient_sampler, uv2, 1, 0.0) * settings.contribute_layers.y; 
    let gradient_3 = textureSampleLevel(gradient_textures, gradient_sampler, uv3, 2, 0.0) * settings.contribute_layers.z; 
    let gradient_4 = textureSampleLevel(gradient_textures, gradient_sampler, uv4, 3, 0.0) * settings.contribute_layers.w; 

    var gradient = gradient_1.xyz + gradient_2.xyz + gradient_3.xyz + gradient_4.xyz;
    gradient *= settings.normal_strength;

    let scene_depth = linearize_depth(in.position.z);

    let macro_normal = vec3(0.0, 1.0, 0.0);
    let normal = normalize(vec3(-gradient.x, 1.0, -gradient.y));

    let directional_light = view_bindings::lights.directional_lights[0u];
    let to_light = normalize(directional_light.direction_to_light);
    var dir = normalize(in.world_position.xyz - view_bindings::view.world_position.xyz);

    let mu = dot(to_light, dir);
    let color = get_ocean_color(in.world_position.xyz, normal, to_light, dir, mu);

    return vec4(color, 1.0);
}