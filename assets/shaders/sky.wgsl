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

const PI: f32 = 3.1415927;

struct SkySettings {
    sun_color: vec3<f32>,
    sun_falloff: f32,
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    var col = textureLoad(screen_texture, vec2<i32>(in.position.xy), 0);
    var view_vector = view.inverse_projection * (vec4(in.uv * 2.0 - 1.0, 0.0, 1.0) * vec4(1.0, -1.0, 1.0, 1.0));
    view_vector = view_vector * view.inverse_view;
    let view_dir = normalize(view_vector.xyz / abs(view_vector.w));

    let directional_light = lights.directional_lights[0u];
    let to_light = normalize(directional_light.direction_to_light);

    let sun = settings.sun_color * pow(saturate(dot(view_dir.xyz, to_light)), settings.sun_falloff);

    return vec4(saturate(col.rgb + sun), 1.0);
}