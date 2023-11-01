use bevy::{prelude::*, render::{render_graph, render_resource::{PipelineCache, ComputePassDescriptor, BindGroupDescriptor, BindGroupEntry, BindingResource}, renderer::RenderContext, render_asset::RenderAssets}};

use super::{pipeline::OceanComputePipeline, uniforms::{OceanComputeTextures, OceanComputeUniforms}, DEFAULT_TEXTURE_SIZE, WORKGROUP_SIZE};



pub struct OceanComputeNode {
    // state: SlimeMoldState,
}

impl OceanComputeNode {
    pub const NAME: &'static str = "ocean_compute_node";
}

impl Default for OceanComputeNode {
    fn default() -> Self {
        Self {
            // state: SlimeMoldState::Loading,
        }
    }
}

impl render_graph::Node for OceanComputeNode {
    fn update(&mut self, _world: &mut World) {

    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let compute_pipeline = world.resource::<OceanComputePipeline>();

        let gpu_images = world.resource::<RenderAssets<Image>>();
        let ocean_textures = world.resource::<OceanComputeTextures>();

        let uniforms = world.resource::<OceanComputeUniforms>();

        let displacement_texture = &gpu_images[&ocean_textures.displacement];

        let bind_group = render_context
            .render_device()
            .create_bind_group(&BindGroupDescriptor {
                label: Some("ocean_compute_pass_bind_group"),
                layout: &compute_pipeline.layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: uniforms.uniforms.binding().unwrap(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&displacement_texture.texture_view),
                    },
                ],
            });

        let encoder = render_context.command_encoder();
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());

            pass.set_bind_group(0, &bind_group, &[]);

            let Some(pipeline) = pipeline_cache.get_compute_pipeline(compute_pipeline.pipeline_id) else {
                return Ok(());
            };
            pass.set_pipeline(pipeline);
            pass.dispatch_workgroups(DEFAULT_TEXTURE_SIZE / WORKGROUP_SIZE, DEFAULT_TEXTURE_SIZE / WORKGROUP_SIZE, 1);
        }

        Ok(())
    }
}