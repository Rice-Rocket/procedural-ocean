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

use super::{uniforms::OceanComputeSettings, spectrums::OceanSpectrumsArray};


#[derive(Resource)]
pub struct OceanComputePipeline {
    pub layout: BindGroupLayout,

    pub init_spectrum_pipeline: CachedComputePipelineId,
    pub pack_spectrum_conj_pipeline: CachedComputePipelineId,
    pub update_spectrum_pipeline: CachedComputePipelineId,
    pub horizontal_fft_pipeline: CachedComputePipelineId,
    pub vertical_fft_pipeline: CachedComputePipelineId,
    pub assemble_maps_pipeline: CachedComputePipelineId,
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
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: Some(OceanSpectrumsArray::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba32Float,
                        view_dimension: TextureViewDimension::D2Array,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rg32Float,
                        view_dimension: TextureViewDimension::D2Array,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba32Float,
                        view_dimension: TextureViewDimension::D2Array,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba32Float,
                        view_dimension: TextureViewDimension::D2Array,
                    },
                    count: None,
                },
            ],
        });

            
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/displacement.wgsl");

        let pipeline_cache = world.resource::<PipelineCache>();
        

        let init_spectrum_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: "initialize_spectrum".into(),
        });
        
        let pack_spectrum_conj_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: "pack_spectrum_conjugates".into(),
        });
        
        let update_spectrum_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: "update_spectrum".into(),
        });
        
        let horizontal_fft_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: "horizontal_fft".into(),
        });
        
        let vertical_fft_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: "vertical_fft".into(),
        });
        
        let assemble_maps_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: "assemble_maps".into(),
        });

        OceanComputePipeline {
            layout, 

            init_spectrum_pipeline,
            pack_spectrum_conj_pipeline,
            update_spectrum_pipeline,
            horizontal_fft_pipeline,
            vertical_fft_pipeline,
            assemble_maps_pipeline,
        }
    }
}