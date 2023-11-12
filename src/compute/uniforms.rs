use bevy::{
    prelude::*, 
    render::{
        render_resource::{
            Extent3d, TextureDimension, TextureFormat, TextureUsages, ShaderType, UniformBuffer, SamplerDescriptor, FilterMode
        }, 
        renderer::{
            RenderDevice, RenderQueue
        }, 
        extract_resource::ExtractResource, texture::ImageSampler
    }
};

use super::{TEXTURE_SIZE, node::OceanInitSpectrumStatus, spectrums::OceanSpectrumsDisplayArray};


#[derive(Clone, Resource, ExtractResource, Reflect, ShaderType)]
#[reflect(Resource)]
pub struct OceanComputeSettings {
    pub lambda: Vec2,
    pub frame_time: f32,
    pub delta_time: f32,
    pub gravity: f32,
    pub repeat_time: f32,
    pub n: u32,
    pub seed: u32,
    pub length_scale_0: u32,
    pub length_scale_1: u32,
    pub length_scale_2: u32,
    pub length_scale_3: u32,
    pub foam_threshold: f32,
    pub depth: f32,
    pub low_cutoff: f32,
    pub high_cutoff: f32,
    pub foam_bias: f32,
    pub foam_decay_rate: f32,
    pub foam_add: f32,

    // #[cfg(all(feature = "webgl", target_arch = "wasm32"))]
    // _webgl2_padding: f32,
}

impl Default for OceanComputeSettings {
    fn default() -> Self {
        Self {
            low_cutoff: 0.0001,
            high_cutoff: 9000.0,
            gravity: 9.81,
            depth: 0.1,
            repeat_time: 200.0,
            frame_time: 1.0,
            lambda: Vec2::new(1.0, 1.0),
            length_scale_0: 128,
            length_scale_1: 64,
            length_scale_2: 32,
            length_scale_3: 16,
            n: TEXTURE_SIZE,
            delta_time: 0.0,
            seed: 0,
            foam_threshold: 0.0,
            foam_bias: 0.0,
            foam_add: 0.0,
            foam_decay_rate: 0.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct OceanComputeUniforms {
    pub buf: UniformBuffer<OceanComputeSettings>,
}

pub fn prepare_uniforms(
    mut uniforms: ResMut<OceanComputeUniforms>,
    general_settings: Res<OceanComputeSettings>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,

    time: Res<Time>,
) {
    let general = uniforms.buf.get_mut();
    *general = general_settings.clone();

    general.n = TEXTURE_SIZE;
    general.frame_time = time.elapsed_seconds() * general_settings.frame_time;
    general.delta_time = time.delta_seconds();

    uniforms.buf.write_buffer(&render_device, &render_queue);
}

pub fn update_init_spectrum_status(
    settings: Res<OceanComputeSettings>,
    spectrum_settings: Res<OceanSpectrumsDisplayArray>,
    mut init_spectrum_status: ResMut<OceanInitSpectrumStatus>,
) {
    if settings.is_changed() || spectrum_settings.is_changed() {
        *init_spectrum_status = OceanInitSpectrumStatus::Update;
    }
}


#[derive(Resource, ExtractResource, Clone)]
pub struct OceanComputeTextures {
    pub displacements: Handle<Image>,
    pub gradients: Handle<Image>,
    pub init_spectrum_textures: Handle<Image>,
    pub spectrum_textures: Handle<Image>,
}

pub fn setup_textures(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let extent = Extent3d {
        width: TEXTURE_SIZE,
        height: TEXTURE_SIZE,
        depth_or_array_layers: 4,
    };
    let mut empty_im_rgba = Image::new_fill(
        extent, 
        TextureDimension::D2, 
        &[0; 16], 
        TextureFormat::Rgba32Float,
    );
    let mut empty_im_rg = Image::new_fill(
        extent,
        TextureDimension::D2,
        &[0; 8],
        TextureFormat::Rg32Float,
    );
    let mut empty_im_rgba_d8 = Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE,
            height: TEXTURE_SIZE,
            depth_or_array_layers: 8,
        },
        TextureDimension::D2,
        &[0; 16],
        TextureFormat::Rgba32Float,
    );

    let usage = TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING; 
    empty_im_rgba.texture_descriptor.usage = usage;
    empty_im_rg.texture_descriptor.usage = usage;
    empty_im_rgba_d8.texture_descriptor.usage = usage;

    let bilinear_sampler = ImageSampler::Descriptor(SamplerDescriptor {
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        ..default()
    });

    let mut displacement_im = empty_im_rgba.clone();
    let mut gradient_im = empty_im_rg;

    displacement_im.sampler_descriptor = bilinear_sampler.clone();
    gradient_im.sampler_descriptor = bilinear_sampler;

    let displacements = images.add(displacement_im);
    let gradients = images.add(gradient_im);
    let init_spectrum_textures = images.add(empty_im_rgba);
    let spectrum_textures = images.add(empty_im_rgba_d8);

    commands.insert_resource(OceanComputeTextures {
        displacements,
        gradients,
        init_spectrum_textures,
        spectrum_textures,
    });
}