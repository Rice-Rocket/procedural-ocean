use bevy::{
    prelude::*, 
    render::{
        render_resource::{
            BindGroupLayout, CachedComputePipelineId, BindGroupLayoutDescriptor, 
            BindGroupLayoutEntry, ShaderStages, BindingType, StorageTextureAccess, 
            TextureFormat, TextureViewDimension, PipelineCache, ComputePipelineDescriptor, BufferBindingType, ShaderType
        }, 
        renderer::RenderDevice, 
    }
};

use super::uniforms::OceanComputeSettings;


#[derive(Resource)]
pub struct OceanComputePipeline {
    pub layout: BindGroupLayout,
    pub pipeline_id: CachedComputePipelineId,
}

impl FromWorld for OceanComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(OceanComputeSettings::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba32Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

            
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/displacement.wgsl");

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: "update".into(),
        });

        OceanComputePipeline {
            layout, 
            pipeline_id,
        }
    }
}