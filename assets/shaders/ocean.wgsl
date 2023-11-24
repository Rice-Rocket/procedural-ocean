#define_import_path ocean::main

#import bevy_pbr::mesh_functions as mesh_functions
#import bevy_pbr::skinning
#import bevy_pbr::morph
#import bevy_pbr::mesh_bindings       mesh
#import bevy_pbr::mesh_vertex_output  MeshVertexOutput
#import bevy_pbr::mesh_view_bindings as view_bindings
#import bevy_pbr::mesh_view_types
#import bevy_pbr::prepass_utils as prepass_utils
#import ocean::sky SkySettings

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
    normal_depth_attenuation: f32,
    foam_depth_attenuation: f32,

    normal_strength: f32,
    specular_normal_strength: f32,

    roughness: f32,
    foam_roughness: f32,

    sun_power: f32,
    scatter_color: vec3<f32>,
    bubble_color: vec3<f32>,
    foam_color: vec3<f32>,
    height_modifier: f32,
    bubble_density: f32,

    wave_peak_scatter_strength: f32,
    scatter_strength: f32,
    scatter_shadow_strength: f32,
    environment_light_strength: f32,

    foam_subtract: f32,

    tile_layers: vec4<f32>,
    contribute_layers: vec4<f32>,
}

// struct SkySettings {
//     sun_color: vec3<f32>,
//     sun_falloff: f32,
// }


@vertex
fn vertex(vertex: Vertex) -> MeshVertexOutput {
    let uv = vertex.uv;

    let uv1 = fract(uv * settings.tile_layers.x);
    let uv2 = fract((uv - 0.5) * settings.tile_layers.y);
    let uv3 = fract((uv - 1.125) * settings.tile_layers.z);
    let uv4 = fract((uv - 1.25) * settings.tile_layers.w);

    var displacement_1 = textureSampleLevel(displacement_textures, displacement_sampler, uv1, 0, 0.0); 
    var displacement_2 = textureSampleLevel(displacement_textures, displacement_sampler, uv2, 1, 0.0); 
    var displacement_3 = textureSampleLevel(displacement_textures, displacement_sampler, uv3, 2, 0.0); 
    var displacement_4 = textureSampleLevel(displacement_textures, displacement_sampler, uv4, 3, 0.0); 
    displacement_1 = vec4(displacement_1.rgb * settings.contribute_layers.x, displacement_1.a);
    displacement_2 = vec4(displacement_2.rgb * settings.contribute_layers.y, displacement_2.a);
    displacement_3 = vec4(displacement_3.rgb * settings.contribute_layers.z, displacement_3.a);
    displacement_4 = vec4(displacement_4.rgb * settings.contribute_layers.w, displacement_4.a);
    var displacement = displacement_1 + displacement_2 + displacement_3 + displacement_4;
    displacement.a += settings.foam_subtract;

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
    out.world_normal = vec3(displacement.a, displacement.y, 0.0);

    return out;
}

const PI: f32 = 3.1415927;

fn linearize_depth(depth: f32) -> f32 {
    let far_plane = 1000.0 - 0.1;
    return mix(1.0, view_bindings::view.projection[3][2] / depth / far_plane, f32(depth > 0.0001));
}

fn get_sky_color(dir: vec3<f32>, sun_dir: vec3<f32>) -> vec3<f32> {
    let sky = textureSample(skybox_texture, skybox_sampler, dir).xyz;
    let sun = sky_settings.sun_color * pow(saturate(dot(dir, sun_dir)), sky_settings.sun_falloff);
    return sky + sun;
}

fn smith_masking_beckmann(h: vec3<f32>, s: vec3<f32>, roughness: f32) -> f32 {
    let h_dot_s = max(0.001, saturate(dot(h, s)));
    let a = h_dot_s / (roughness * sqrt(1.0 - h_dot_s * h_dot_s));
    let a2 = a * a;

    if (a < 1.6) {
        return (1.0 - 1.259 * a + 0.396 * a2) / (3.535 * a + 2.181 * a2);
    }
    return 0.0;
}

fn beckmann(n_dot_h: f32, roughness: f32) -> f32 {
    let expo = (n_dot_h * n_dot_h - 1.0) / (roughness * roughness * n_dot_h * n_dot_h);
    return exp(expo) / (PI * roughness * roughness * n_dot_h * n_dot_h * n_dot_h * n_dot_h);
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
    let specular_gradient = gradient * settings.specular_normal_strength;
    gradient *= settings.normal_strength;

    let sun_irradiance = sky_settings.sun_color * settings.sun_power;
    let directional_light = view_bindings::lights.directional_lights[0u];

    let light_dir = normalize(directional_light.direction_to_light);
    let view_dir = normalize(view_bindings::view.world_position.xyz - in.world_position.xyz);
    let half_dir = normalize(light_dir + view_dir);

    let depth = linearize_depth(in.position.z);
    let l_dot_h = saturate(dot(light_dir, half_dir));
    let v_dot_h = saturate(dot(view_dir, half_dir));

    let macro_normal = vec3(0.0, 1.0, 0.0);
    var normal = normalize(vec3(-gradient.x, 1.0, -gradient.y));
    var specular_normal = normalize(vec3(-specular_gradient.x, 1.0, -specular_gradient.y));
    normal = normalize(mix(macro_normal, normal, pow(saturate(depth), settings.normal_depth_attenuation)));
    specular_normal = normalize(mix(macro_normal, specular_normal, pow(saturate(depth), settings.normal_depth_attenuation)));

    var foam = in.world_normal.x;
    foam = mix(0.0, saturate(foam), pow(depth, settings.foam_depth_attenuation));

    let n_dot_l = saturate(dot(normal, light_dir));

    let a = settings.roughness + saturate(foam) * settings.foam_roughness;
    let n_dot_h = max(0.0001, dot(normal, half_dir));
    
    let view_mask = smith_masking_beckmann(half_dir, view_dir, a);
    let light_mask = smith_masking_beckmann(half_dir, light_dir, a);

    let g = 1.0 / (1.0 + view_mask + light_mask);

    let eta = 1.33;
    let r = ((eta - 1.0) * (eta - 1.0)) / ((eta + 1.0) * (eta + 1.0));
    let theta_v = acos(view_dir.y);

    let numerator = pow(1.0 - dot(normal, view_dir), 5.0 * exp(-2.69 * a));
    var f = r + (1.0 - r) * numerator / (1.0 + 22.7 * pow(a, 1.5));
    f = saturate(f);

    var specular = sun_irradiance * f * g * beckmann(max(0.0001, dot(specular_normal, half_dir)), a);
    specular /= 4.0 * max(0.001, saturate(dot(macro_normal, light_dir)));
    specular *= saturate(dot(normal, light_dir));

    var env_reflection = get_sky_color(reflect(-view_dir, normal), light_dir);
    env_reflection *= settings.environment_light_strength;

    let h = max(0.0, in.world_normal.y) * settings.height_modifier;

    let k1 = settings.wave_peak_scatter_strength * h * pow(saturate(dot(light_dir, -view_dir)), 4.0) * pow(0.5 - 0.5 * dot(light_dir, normal), 3.0);
    let k2 = settings.scatter_strength * pow(saturate(dot(view_dir, normal)), 2.0);
    let k3 = settings.scatter_shadow_strength * n_dot_l;
    let k4 = settings.bubble_density;

    var scatter = (k1 + k2) * settings.scatter_color * sun_irradiance / (1.0 + light_mask);
    scatter += k3 * settings.scatter_color * sun_irradiance + k4 * settings.bubble_color * sun_irradiance;

    var output = /* (1.0 - f) * */scatter + specular + f * env_reflection;
    output = max(vec3(0.0), output);
    output = mix(output, settings.foam_color, saturate(foam));

    return vec4(output, 1.0);
}