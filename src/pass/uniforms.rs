use bevy::{
    prelude::*, 
    render::{
        render_resource::{
            Extent3d, TextureDimension, TextureFormat, TextureUsages, ShaderType, UniformBuffer
        }, 
        renderer::{
            RenderDevice, RenderQueue
        }, 
        extract_resource::ExtractResource
    }
};

use super::DEFAULT_TEXTURE_SIZE;


#[derive(Default, Clone, Resource, ExtractResource, Reflect, ShaderType)]
#[reflect(Resource)]
pub struct OceanComputeSettings {
    pub dimensions: Vec2,
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
    pub normal_strength: f32,
    pub foam_threshold: f32,
    pub depth: f32,
    pub low_cutoff: f32,
    pub high_cutoff: f32,
    pub foam_bias: f32,
    pub foam_decay_rate: f32,
    pub foam_add: f32,

    // #[cfg(all(feature = "webgl", target_arch = "wasm32"))]
    // _webgl2_padding: Vec2,
}

#[derive(Resource, Default)]
pub struct OceanComputeUniforms {
    pub uniforms: UniformBuffer<OceanComputeSettings>,
}

pub fn prepare_uniforms(
    mut uniforms: ResMut<OceanComputeUniforms>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,

    time: Res<Time>,
) {
    let buf = uniforms.uniforms.get_mut();

    buf.dimensions = Vec2::splat(DEFAULT_TEXTURE_SIZE as f32);
    buf.frame_time = time.elapsed_seconds();
    buf.delta_time = time.delta_seconds();

    uniforms.uniforms.write_buffer(&render_device, &render_queue);
}


#[derive(Resource, ExtractResource, Clone)]
pub struct OceanComputeTextures {
    pub displacement: Handle<Image>,
}

pub fn setup_textures(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let extent = Extent3d {
        width: DEFAULT_TEXTURE_SIZE,
        height: DEFAULT_TEXTURE_SIZE,
        depth_or_array_layers: 1,
    };
    let mut empty_im = Image::new_fill(extent, 
        TextureDimension::D2, 
        &[0; 16], 
        TextureFormat::Rgba32Float
    );

    empty_im.texture_descriptor.usage = 
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING; 

    let displacement = images.add(empty_im);

    commands.insert_resource(OceanComputeTextures {
        displacement,
    });
}