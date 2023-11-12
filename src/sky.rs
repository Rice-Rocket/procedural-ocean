use bevy::{
    core_pipeline::{fullscreen_vertex_shader::fullscreen_shader_vertex_state, core_3d},
    prelude::*,
    render::{
        extract_component::{ComponentUniforms, ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin},
        render_graph::{NodeRunError, RenderGraphContext, ViewNode, ViewNodeRunner, RenderGraphApp},
        render_resource::{
            BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
            BindGroupLayoutEntry, BindingResource, BindingType, CachedRenderPipelineId,
            ColorTargetState, ColorWrites, FragmentState, MultisampleState, Operations,
            PipelineCache, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
            RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
            ShaderType, TextureFormat, TextureSampleType, TextureViewDimension, BufferBindingType,
        },
        renderer::{RenderContext, RenderDevice},
        texture::BevyDefault,
        view::{ViewTarget, ViewUniforms, ViewUniform, ViewUniformOffset}, RenderApp
    },
    ecs::query::QueryItem, pbr::{GpuLights, LightMeta, ViewLightsUniformOffset},
};


#[derive(Default)]
pub struct SkyPassPostProcessNode;
impl SkyPassPostProcessNode {
    pub const NAME: &str = "sky_pass_post_process";
}

impl ViewNode for SkyPassPostProcessNode {
    type ViewQuery = (
        &'static ViewTarget, 
        bevy::ecs::system::lifetimeless::Read<ViewUniformOffset>,
        bevy::ecs::system::lifetimeless::Read<ViewLightsUniformOffset>
    );
    
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        view_target: QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let post_process_pipeline = world.resource::<SkyPassPostProcessPipeline>();

        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id) else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<SkyPostProcessSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let Some(view_binding) = world.resource::<ViewUniforms>().uniforms.binding() else {
            return Ok(());
        };

        let Some(lights_binding) = world.resource::<LightMeta>().view_gpu_lights.binding() else {
            return Ok(());
        };

        let post_process = view_target.0.post_process_write();

        let bind_group = render_context
            .render_device()
            .create_bind_group(&BindGroupDescriptor {
                label: Some("sky_pass_post_process_bind_group"),
                layout: &post_process_pipeline.layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(post_process.source),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&post_process_pipeline.sampler),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: settings_binding.clone(),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: view_binding.clone(),
                    },
                    BindGroupEntry {
                        binding: 4,
                        resource: lights_binding.clone(),
                    },
                ],
            });

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("sky_pass_post_process_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[view_target.1.offset, view_target.2.offset]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
pub struct SkyPassPostProcessPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for SkyPassPostProcessPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("sky_pass_post_process_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: bevy::render::render_resource::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(SkyPostProcessSettings::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(ViewUniform::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(GpuLights::min_size()),
                    },
                    count: None,
                },
            ],
        });

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let shader = world
            .resource::<AssetServer>()
            .load("shaders/sky.wgsl");

        let pipeline_id = world
            .resource_mut::<PipelineCache>()
            .queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("sky_pass_post_process_pipeline".into()),
                layout: vec![layout.clone()],
                vertex: fullscreen_shader_vertex_state(),
                fragment: Some(FragmentState {
                    shader,
                    shader_defs: vec![],
                    entry_point: "fragment".into(),
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::bevy_default(),
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                push_constant_ranges: vec![],
            });

        Self {
            layout,
            sampler,
            pipeline_id,
        }
    }
}


#[derive(Component, Debug, Clone, Copy, ExtractComponent, ShaderType, Reflect)]
#[reflect(Debug, Default)]
pub struct SkyPostProcessSettings {
    pub sun_color: Vec3,
    pub sun_falloff: f32,
}

impl Default for SkyPostProcessSettings {
    fn default() -> Self {
        Self {
            sun_color: Vec3::new(1.0, 1.0, 1.0),
            sun_falloff: 3500.0,
        }
    }
}

pub struct SkyPostProcessPlugin;

impl Plugin for SkyPostProcessPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<SkyPostProcessSettings>::default(),
            UniformComponentPlugin::<SkyPostProcessSettings>::default(),
        ));

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_render_graph_node::<ViewNodeRunner<SkyPassPostProcessNode>>(
                core_3d::graph::NAME,
                SkyPassPostProcessNode::NAME,
            )
            .add_render_graph_edges(
                core_3d::graph::NAME,
                &[
                    core_3d::graph::node::TONEMAPPING,
                    SkyPassPostProcessNode::NAME,
                    core_3d::graph::node::END_MAIN_PASS_POST_PROCESSING,
                ],
            );
    }

    fn finish(&self, app: &mut App) {
        app.register_type::<SkyPostProcessSettings>();

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<SkyPassPostProcessPipeline>();
    }
}