#define_import_path ocean::sky

#import bevy_core_pipeline::fullscreen_vertex_shader FullscreenVertexOutput
#import bevy_pbr::mesh_view_types as pbr_types
#import bevy_render::view View

@group(0) @binding(0)
var screen_texture: texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler: sampler;
@group(0) @binding(2)
var<uniform> settings: SkySettings;
@group(0) @binding(3)
var<uniform> view: View;
@group(0) @binding(4)
var<uniform> lights: pbr_types::Lights;
@group(0) @binding(5)
var skybox_texture: texture_cube<f32>;
@group(0) @binding(6)
var depth_texture: texture_depth_2d;

const PI: f32 = 3.1415927;

struct SkySettings {
    sun_color: vec3<f32>,
    sun_falloff: f32,
    fog_color: vec3<f32>,
    fog_density: f32,
    fog_offset: f32,
    fog_height: f32,
    fog_attenuation: f32,
    skybox_speed: f32,
}

fn linearize_depth_01(depth: f32) -> f32 {
    let far_plane = 1000.0 - 0.1;
    return mix(1.0, view.projection[3][2] / depth / far_plane, f32(depth > 0.0001));
}

fn linearize_depth(depth: f32) -> f32 {
    return view.projection[3][2] / depth;
}

fn get_sky_color(dir: vec3<f32>, sun_dir: vec3<f32>) -> vec3<f32> {
    let sky = textureSample(skybox_texture, texture_sampler, dir).xyz;
    return sky;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    var color = textureLoad(screen_texture, vec2<i32>(in.position.xy), 0);
    var view_vector = view.inverse_projection * (vec4(in.uv * 2.0 - 1.0, 0.0, 1.0) * vec4(1.0, -1.0, 1.0, 1.0));
    view_vector = view_vector * view.inverse_view;
    let view_dir = normalize(view_vector.xyz / abs(view_vector.w));

    let directional_light = lights.directional_lights[0u];
    let to_light = normalize(directional_light.direction_to_light);

    let depth = textureSample(depth_texture, texture_sampler, in.uv);

    if (depth < 0.0001) {
        color = vec4(get_sky_color(view_dir, to_light), 1.0);
    }

    var view_dist = linearize_depth(depth);
    if (depth < 0.0001) {
        view_dist = 1000.0;
    }
    
    let world_pos = view.world_position.xyz + view_dir * view_dist;
    var height = min(settings.fog_height, world_pos.y) / settings.fog_height;
    height = pow(saturate(height), 1.0 / settings.fog_attenuation);

    var fog_factor = (settings.fog_density / sqrt(log(2.0))) * max(0.0, view_dist - settings.fog_offset);
    fog_factor = exp2(-fog_factor * fog_factor);

    let sun = settings.sun_color * pow(saturate(dot(view_dir, to_light)), settings.sun_falloff);
    
    let output = mix(settings.fog_color, color.rgb, saturate(height + fog_factor));

    return vec4(output + sun, 1.0);
}