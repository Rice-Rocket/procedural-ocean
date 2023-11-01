@group(0) @binding(0)
var<uniform> settings: OceanSettings;
@group(0) @binding(1)
var displacement_texture: texture_storage_2d<rgba32float, read_write>;

struct OceanSettings {
    dimensions: vec2<f32>,
    lambda: vec2<f32>,
    frame_time: f32,
    delta_time: f32,
    gravity: f32,
    repeat_time: f32,
    n: u32,
    seed: u32,
    length_scale_0: u32,
    length_scale_1: u32,
    length_scale_2: u32,
    length_scale_3: u32,
    normal_strength: f32,
    foam_threshold: f32,
    depth: f32,
    low_cutoff: f32,
    high_cutoff: f32,
    foam_bias: f32,
    foam_decay_rate: f32,
    foam_add: f32,

#ifdef SIXTEEN_BYTE_ALIGNMENT
    _webgl2_padding: vec2<f32>
#endif
}


@compute @workgroup_size(32, 32, 1)
fn update(@builtin(global_invocation_id) id: vec3<u32>) {
    let location = vec2<i32>(i32(id.x), i32(id.y));
    let uv = vec2<f32>(location) / settings.dimensions;

    let height = sin(uv.x + uv.y + settings.frame_time);

    storageBarrier();
    textureStore(displacement_texture, location, vec4(height));
}