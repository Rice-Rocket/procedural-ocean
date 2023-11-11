@group(0) @binding(0)
var<uniform> settings: OceanSettings;
@group(0) @binding(1)
var<storage, read_write> spectrums: array<OceanSpectrumSettings, 8>;
@group(0) @binding(2)
var displacement_textures: texture_storage_2d_array<rgba32float, read_write>;
@group(0) @binding(3)
var gradient_textures: texture_storage_2d_array<rg32float, write>;
@group(0) @binding(4)
var init_spectrum_textures: texture_storage_2d_array<rgba32float, read_write>;
@group(0) @binding(5)
var spectrum_textures: texture_storage_2d_array<rgba32float, read_write>;

struct OceanSettings {
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
    foam_threshold: f32,
    depth: f32,
    low_cutoff: f32,
    high_cutoff: f32,
    foam_bias: f32,
    foam_decay_rate: f32,
    foam_add: f32,

#ifdef SIXTEEN_BYTE_ALIGNMENT
    _webgl_padding: f32,
#endif
}

struct OceanSpectrumSettings {
    scale: f32,
    angle: f32,
    spread_blend: f32,
    swell: f32,
    alpha: f32,
    peak_omega: f32,
    gamma: f32,
    short_waves_fade: f32,
}

const PI: f32 = 3.1415927;
const TAU: f32 = 6.2831853;

fn complex_mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

fn complex_exp(a: vec2<f32>) -> vec2<f32> {
    return vec2(cos(a.y), sin(a.y) * exp(a.x));
}

fn euler_formula(x: f32) -> vec2<f32> {
    return vec2(cos(x), sin(x));
}

fn hash(state: u32) -> f32 {
    var n = (state << 13u) ^ state;
    n = n * (n * n * 15731u + 789221u) + 1376312589u;
    return f32(n & u32(0x7fffffffu)) / f32(0x7fffffff);
}

fn uniform_to_gauss(u1: f32, u2: f32) -> vec2<f32> {
    let r = sqrt(-2.0 * log(u1));
    let theta = TAU * u2;
    return vec2(r * cos(theta), r * sin(theta));
}

fn dispersion(k_mag: f32) -> f32 {
    return sqrt(settings.gravity * k_mag * tanh(min(k_mag * settings.depth, 20.0)));
}

fn dispersion_derivative(k_mag: f32) -> f32 {
    let th = tanh(min(k_mag * settings.depth, 20.0));
    let ch = cosh(k_mag * settings.depth);
    return settings.gravity * (settings.depth * k_mag / ch / ch + th) / dispersion(k_mag) / 2.0;
}

fn normalization_factor(s: f32) -> f32 {
    let s2 = s * s;
    let s3 = s2 * s;
    let s4 = s3 * s;
    if (s < 5.0) {
        return -0.000564 * s4 + 0.00776 * s3 - 0.044 * s2 + 0.192 * s + 0.163; 
    } else {
        return -4.80e-08 * s4 + 1.07e-05 * s3 - 9.53e-04 * s2 + 5.90e-02 * s + 3.93e-01;
    }
}

fn donelan_banner_beta(x: f32) -> f32 {
    if (x < 0.95) {
        return 2.61 * pow(abs(x), 1.3);
    }
    if (x < 1.6) {
        return 2.28 * pow(abs(x), -1.3);
    }

    let p = -0.4 + 0.8393 * exp(-0.567 * log(x * x));
    return pow(10.0, p);
}

fn donelan_banner(theta: f32, omega: f32, peak_omega: f32) -> f32 {
    let beta = donelan_banner_beta(omega / peak_omega);
    let sech = 1.0 / cosh(beta * theta);
    return beta / 2.0 / tanh(beta * PI) * sech * sech;
}

fn cosine_2s(theta: f32, s: f32) -> f32 {
    return normalization_factor(s) * pow(abs(cos(0.5 * theta)), 2.0 * s);
}

fn spread_power(omega: f32, peak_omega: f32) -> f32 {
    if (omega > peak_omega) {
        return 9.77 * pow(abs(omega / peak_omega), -2.5);
    } else {
        return 6.97 * pow(abs(omega / peak_omega), 5.0);
    }
}

fn direction_spectrum(theta: f32, omega: f32, spectrum: OceanSpectrumSettings) -> f32 {
    let c_theta = cos(theta);
    let s = spread_power(omega, spectrum.peak_omega) + 16.0 * tanh(min(omega / spectrum.peak_omega, 20.0)) * spectrum.swell * spectrum.swell;
    return mix(2.0 / PI * c_theta * c_theta, cosine_2s(theta - spectrum.angle, s), spectrum.spread_blend);
}

fn tma_correction(omega: f32) -> f32 {
    let omega_h = omega * sqrt(settings.depth / settings.gravity);
    if (omega_h <= 1.0) {
        return 0.5 * omega_h * omega_h;
    }
    if (omega_h < 2.0) {
        return 1.0 - 0.5 * (2.0 - omega_h) * (2.0 - omega_h);
    }
    return 1.0;
}

fn jonswap(omega: f32, spectrum: OceanSpectrumSettings) -> f32 {
    let sigma = mix(0.09, 0.07, f32(omega <= spectrum.peak_omega));

    let r = exp(-(omega - spectrum.peak_omega) * (omega - spectrum.peak_omega) / 2.0 / sigma / sigma / spectrum.peak_omega / spectrum.peak_omega);

    let inv_omega = 1.0 / omega;
    let peak_omega_over_omega = spectrum.peak_omega / omega;

    return spectrum.scale * tma_correction(omega) * spectrum.alpha * settings.gravity * settings.gravity
        * inv_omega * inv_omega * inv_omega * inv_omega * inv_omega
        * exp(-1.25 * peak_omega_over_omega * peak_omega_over_omega * peak_omega_over_omega * peak_omega_over_omega)
        * pow(abs(spectrum.gamma), r);
}

fn short_waves_fade(k_length: f32, spectrum: OceanSpectrumSettings) -> f32 {
    return exp(-spectrum.short_waves_fade * spectrum.short_waves_fade * k_length * k_length);
}


@compute @workgroup_size(8, 8, 1)
fn initialize_spectrum(@builtin(global_invocation_id) id: vec3<u32>) {
    let location = vec2<f32>(id.xy);

    var seed = id.x + settings.n * id.y + settings.n + settings.seed;
    let half_n = f32(settings.n) / 2.0;

    let length_scales = vec4<u32>(settings.length_scale_0, settings.length_scale_1, settings.length_scale_2, settings.length_scale_3);

    for (var i = 0u; i < 4u; i++) {
        let delta_k = TAU / f32(length_scales[i]);
        let k = (location - half_n) * delta_k;
        let k_length = length(k);

        seed += i + u32(hash(seed)) * 10u;

        let uniform_rand_samples = vec4(hash(seed * 2u), hash(seed * 3u), hash(seed * 5u), hash(seed * 7u));
        let gauss_1 = uniform_to_gauss(uniform_rand_samples.x, uniform_rand_samples.y);
        let gauss_2 = uniform_to_gauss(uniform_rand_samples.z, uniform_rand_samples.w);

        var storage_value = vec4(0.0);
        if (settings.low_cutoff <= k_length && k_length <= settings.high_cutoff) {
            let k_angle = atan2(k.y, k.x);
            let omega = dispersion(k_length);

            let d_omega_dk = dispersion_derivative(k_length);

            var spectrum = jonswap(omega, spectrums[i * 2u]) * direction_spectrum(k_angle, omega, spectrums[i * 2u]) * short_waves_fade(k_length, spectrums[i * 2u]);

            if (spectrums[i * 2u + 1u].scale > 0.0) {
                spectrum += jonswap(omega, spectrums[i * 2u + 1u]) * direction_spectrum(k_angle, omega, spectrums[i * 2u + 1u]) * short_waves_fade(k_length, spectrums[i * 2u + 1u]);
            }

            storage_value = vec4(vec2(gauss_2.x, gauss_1.y) * sqrt(2.0 * spectrum * abs(d_omega_dk) / k_length * delta_k * delta_k), 0.0, 0.0);
        }

        // storageBarrier();
        textureStore(init_spectrum_textures, id.xy, i, storage_value);
    }
}

@compute @workgroup_size(8, 8, 1)
fn pack_spectrum_conjugates(@builtin(global_invocation_id) id: vec3<u32>) {
    let loc = vec2<i32>(id.xy);
    let n = i32(settings.n);
    for (var i = 0u; i < 4u; i++) {
        let h0 = textureLoad(init_spectrum_textures, id.xy, i).xy;
        let conj_pos = vec2((n - loc.x) % n, (n - loc.y) % n);
        let conj = textureLoad(init_spectrum_textures, conj_pos, i).xy;

        // storageBarrier();
        textureStore(init_spectrum_textures, id.xy, i, vec4(h0, conj.x, -conj.y));
    }
}

@compute @workgroup_size(8, 8, 1)
fn update_spectrum(@builtin(global_invocation_id) id: vec3<u32>) {
    let length_scales = vec4<u32>(settings.length_scale_0, settings.length_scale_1, settings.length_scale_2, settings.length_scale_3);
    let half_n = f32(settings.n) / 2.0;
    let location = vec2<f32>(id.xy);

    for (var i = 0u; i < 4u; i++) {
        let init_signal = textureLoad(init_spectrum_textures, id.xy, i);
        let h0 = init_signal.xy;
        let h0_conj = init_signal.zw;

        let k = (location - half_n) * TAU / f32(length_scales[i]);
        let k_mag = length(k);
        var k_mag_rcp = 1.0 / max(0.0001, k_mag);

        if (k_mag < 0.0001) {
            k_mag_rcp = 1.0;
        }

        let w_0 = TAU / settings.repeat_time;
        let dispersion = floor(sqrt(settings.gravity * k_mag) / w_0) * w_0 * settings.frame_time;

        let exponent = euler_formula(dispersion);

        let h_tilde = complex_mul(h0, exponent) + complex_mul(h0_conj, vec2(exponent.x, -exponent.y));
        let ih = vec2(-h_tilde.y, h_tilde.x);

        let displacement_x = ih * k.x * k_mag_rcp;
        let displacement_y = h_tilde;
        let displacement_z = ih * k.y * k_mag_rcp;

        let displacement_x_dx = -h_tilde * k.x * k.x * k_mag_rcp;
        let displacement_y_dx = ih * k.x;
        let displacement_z_dx = -h_tilde * k.x * k.y * k_mag_rcp;

        let displacement_y_dz = ih * k.y;
        let displacement_z_dz = -h_tilde * k.y * k.y * k_mag_rcp;

        let h_tilde_displacement_x = vec2(displacement_x.x - displacement_z.y, displacement_x.y + displacement_z.x);
        let h_tilde_displacement_z = vec2(displacement_y.x - displacement_z_dx.y, displacement_y.y + displacement_z_dx.x);

        let h_tilde_grad_x = vec2(displacement_y_dx.x - displacement_y_dz.y, displacement_y_dx.y + displacement_y_dz.x);
        let h_tilde_grad_z = vec2(displacement_x_dx.x - displacement_z_dz.y, displacement_x_dx.y + displacement_z_dz.x);

        // storageBarrier();
        textureStore(spectrum_textures, id.xy, i * 2u, vec4(h_tilde_displacement_x, h_tilde_displacement_z));
        textureStore(spectrum_textures, id.xy, i * 2u + 1u, vec4(h_tilde_grad_x, h_tilde_grad_z));
    }
}

const SIZE: u32 = 256u;
const LOG_SIZE: u32 = 8u;
// const SIZE: u32 = 1024u;
// const LOG_SIZE: u32 = 10u;

fn twiddle_factor_and_input_indices(id: vec2<u32>) -> vec4<f32> {
    let b = settings.n >> (id.x + 1u);
    let mul = TAU * vec2(0.0, 1.0) / f32(settings.n);
    let i = (2u * b * (id.y / b) + id.y % b) % settings.n;
    let twiddle = complex_exp(-mul * f32((id.y / b) * b));

    return vec4(twiddle, f32(i), f32(i + b));
}

struct ButterflyValuesResult {
    twiddle: vec2<f32>,
    indices: vec2<u32>,
}

fn butterfly_values(step: u32, idx: u32) -> ButterflyValuesResult {
    let b = SIZE >> (step + 1u);
    let w = b * (idx / b);
    let i = (w + idx) % SIZE;
    let theta = -TAU / f32(SIZE) * f32(w);

    let twiddle = vec2(cos(theta), -sin(theta));
    let indices = vec2(i, i + b);
    return ButterflyValuesResult(twiddle, indices);
}

var<workgroup> fft_group_buffer: array<array<vec4<f32>, SIZE>, 2>;

fn fft(thread_idx: u32, input: vec4<f32>) -> vec4<f32> {
    fft_group_buffer[0][thread_idx] = input;
    workgroupBarrier();
    
    var flag = 0;

    for (var step = 0u; step < LOG_SIZE; step++) {
        let twiddle_indices = butterfly_values(step, thread_idx);
        let twiddle = twiddle_indices.twiddle;
        let indices = twiddle_indices.indices;

        let v = fft_group_buffer[flag][indices.y];
        fft_group_buffer[1 - flag][thread_idx] = fft_group_buffer[flag][indices.x] + vec4(complex_mul(twiddle, v.xy), complex_mul(twiddle, v.zw));

        flag = 1 - flag;
        workgroupBarrier();
    }

    return fft_group_buffer[flag][thread_idx];
}

@compute @workgroup_size(256, 1, 1)
// @compute @workgroup_size(1024, 1, 1)
fn horizontal_fft(@builtin(global_invocation_id) id: vec3<u32>) {
    for (var i = 0u; i < 8u; i++) {
        let old = textureLoad(spectrum_textures, id.xy, i);
        textureStore(spectrum_textures, id.xy, i, fft(id.x, old));
    }
}

@compute @workgroup_size(256, 1, 1)
// @compute @workgroup_size(1024, 1, 1)
fn vertical_fft(@builtin(global_invocation_id) id: vec3<u32>) {
    for (var i = 0u; i < 8u; i++) {
        let old = textureLoad(spectrum_textures, id.yx, i);
        textureStore(spectrum_textures, id.yx, i, fft(id.x, old));
    }
}

fn permute(data: vec4<f32>, id: vec2<f32>) -> vec4<f32> {
    // return data;
    return data * (1.0 - 2.0 * ((id.x + id.y) % 2.0));
}

@compute @workgroup_size(8, 8, 1)
fn assemble_maps(@builtin(global_invocation_id) id: vec3<u32>) {
    for (var i = 0u; i < 4u; i++) {
        let h_tilde_displacement = permute(textureLoad(spectrum_textures, id.xy, i * 2u), vec2<f32>(id.xy));
        let h_tilde_slope = permute(textureLoad(spectrum_textures, id.xy, i * 2u + 1u), vec2<f32>(id.xy));

        let dxdz = h_tilde_displacement.xy;
        let dydxz = h_tilde_displacement.zw;
        let dyxdyz = h_tilde_slope.xy;
        let dxxdzz = h_tilde_slope.zw;

        let jacobian = (1.0 + settings.lambda.x * dxxdzz.x) * (1.0 + settings.lambda.y * dxxdzz.y) - settings.lambda.x * settings.lambda.y * dydxz.y * dydxz.y;

        let displacement = vec3(settings.lambda.x * dxdz.x, dydxz.x, settings.lambda.y * dxdz.y);
        
        let gradients = dyxdyz.xy / (1.0 + abs(dxxdzz * settings.lambda));
        let covariance = gradients.x * gradients.y;

        // storageBarrier();
        textureStore(displacement_textures, id.xy, i, vec4(displacement, 1.0));
        textureStore(gradient_textures, id.xy, i, vec4(gradients, 0.0, 0.0));
    }
}