use bevy::{
    prelude::*, 
    render::{
        render_resource::{
            ShaderType, StorageBuffer
        }, 
        renderer::{
            RenderDevice, RenderQueue
        }, 
        extract_resource::ExtractResource
    }
};

use super::uniforms::OceanComputeSettings;

#[derive(Default, Clone, Resource, ExtractResource, Reflect, ShaderType)]
#[reflect(Resource)]
pub struct OceanSpectrumSettings {
    pub scale: f32,
    pub angle: f32,
    pub spread_blend: f32,
    pub swell: f32,
    pub alpha: f32,
    pub peak_omega: f32,
    pub gamma: f32,
    pub short_waves_fade: f32,
}

#[derive(Default, Clone, Resource, ExtractResource, Reflect, ShaderType)]
#[reflect(Resource)]
pub struct OceanSpectrumDisplaySettings {
    pub scale: f32,
    pub angle: f32,
    pub spread_blend: f32,
    pub swell: f32,
    pub fetch: f32,
    pub peak_enhancement: f32,
    pub short_waves_fade: f32,
    pub wind_speed: f32,
}

#[derive(Resource, Default, ShaderType, ExtractResource, Reflect, Clone)]
#[reflect(Resource)]
pub struct OceanSpectrumsArray {
    pub spectrums: [OceanSpectrumSettings; 8],
}

#[derive(Resource, ShaderType, ExtractResource, Reflect, Clone)]
#[reflect(Resource)]
pub struct OceanSpectrumsDisplayArray {
    pub spectrums: [OceanSpectrumDisplaySettings; 8],
}

impl Default for OceanSpectrumsDisplayArray {
    fn default() -> Self {
        Self {
            spectrums: [
                OceanSpectrumDisplaySettings {
                    scale: 0.1, 
                    angle: 22.0,
                    spread_blend: 1.0,
                    swell: 0.42,
                    fetch: 100000000.0,
                    peak_enhancement: 1.0,
                    short_waves_fade: 1.0,
                    wind_speed: 20.0,
                },
                OceanSpectrumDisplaySettings {
                    scale: 0.1, 
                    angle: 59.0,
                    spread_blend: 1.0,
                    swell: 1.0,
                    fetch: 1000000.0,
                    peak_enhancement: 1.0,
                    short_waves_fade: 1.0,
                    wind_speed: 24.9,
                },
                OceanSpectrumDisplaySettings {
                    scale: 0.05, 
                    angle: 97.0,
                    spread_blend: 0.14,
                    swell: 1.0,
                    fetch: 100000000.0,
                    peak_enhancement: 1.0,
                    short_waves_fade: 0.5,
                    wind_speed: 20.0,
                },
                OceanSpectrumDisplaySettings {
                    scale: 0.05, 
                    angle: 67.0,
                    spread_blend: 0.47,
                    swell: 1.0,
                    fetch: 1000000.0,
                    peak_enhancement: 1.0,
                    short_waves_fade: 0.5,
                    wind_speed: 20.0,
                },
                OceanSpectrumDisplaySettings {
                    scale: 0.025, 
                    angle: 105.0,
                    spread_blend: 0.2,
                    swell: 1.0,
                    fetch: 1000000.0,
                    peak_enhancement: 1.0,
                    short_waves_fade: 0.5,
                    wind_speed: 5.0,
                },
                OceanSpectrumDisplaySettings {
                    scale: 0.05, 
                    angle: 19.0,
                    spread_blend: 0.298,
                    swell: 0.695,
                    fetch: 10000.0,
                    peak_enhancement: 1.0,
                    short_waves_fade: 0.5,
                    wind_speed: 1.0,
                },
                OceanSpectrumDisplaySettings {
                    scale: 0.0125, 
                    angle: 73.0,
                    spread_blend: 0.1,
                    swell: 1.0,
                    fetch: 1000000.0,
                    peak_enhancement: 1.0,
                    short_waves_fade: 0.5,
                    wind_speed: 3.0,
                },
                OceanSpectrumDisplaySettings {
                    scale: 0.025, 
                    angle: 82.0,
                    spread_blend: 0.15,
                    swell: 0.695,
                    fetch: 10000.0,
                    peak_enhancement: 1.0,
                    short_waves_fade: 0.5,
                    wind_speed: 1.0,
                },
            ],
        }
    }
}

#[derive(Resource, Default)]
pub struct OceanSpectrumStorage {
    pub buf: StorageBuffer<OceanSpectrumsArray>,
}

fn jonswap_alpha(fetch: f32, wind_speed: f32, gravity: f32) -> f32 {
    return 0.076 * (gravity * fetch / wind_speed / wind_speed).powf(-0.22);
}

fn jonswap_peak_freq(fetch: f32, wind_speed: f32, gravity: f32) -> f32 {
    return 22.0 * (wind_speed * fetch / gravity / gravity).powf(-0.33);
}

pub fn prepare_storage(
    mut storage: ResMut<OceanSpectrumStorage>,
    spectrums_arr: Res<OceanSpectrumsDisplayArray>,
    spectrum_uniform: Res<OceanComputeSettings>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let spectrums = storage.buf.get_mut();

    for (i, spectrum) in spectrums.spectrums.iter_mut().enumerate() {
        let display_spec = &spectrums_arr.spectrums[i];
        spectrum.scale = display_spec.scale;
        spectrum.angle = display_spec.angle / 180.0 * std::f32::consts::PI;
        spectrum.spread_blend = display_spec.spread_blend;
        spectrum.swell = display_spec.swell.clamp(0.01, 1.0);
        spectrum.alpha = jonswap_alpha(display_spec.fetch, display_spec.wind_speed, spectrum_uniform.gravity);
        spectrum.peak_omega = jonswap_peak_freq(display_spec.fetch, display_spec.wind_speed, spectrum_uniform.gravity);
        spectrum.gamma = display_spec.peak_enhancement;
        spectrum.short_waves_fade = display_spec.short_waves_fade;
    }
    
    storage.buf.write_buffer(&render_device, &render_queue);
}