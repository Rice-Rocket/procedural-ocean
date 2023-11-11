use bevy::{prelude::*, render::{render_graph, render_resource::{PipelineCache, ComputePassDescriptor, BindGroupDescriptor, BindGroupEntry, BindingResource}, renderer::RenderContext, render_asset::RenderAssets}};

use super::{pipeline::OceanComputePipeline, uniforms::{OceanComputeTextures, OceanComputeUniforms}, TEXTURE_SIZE, WORKGROUP_SIZE, spectrums::OceanSpectrumStorage};



pub struct OceanComputeNode {
    update_spectrum: bool,
    // state: SlimeMoldState,
}

impl OceanComputeNode {
    pub const NAME: &'static str = "ocean_compute_node";
}

impl Default for OceanComputeNode {
    fn default() -> Self {
        Self {
            update_spectrum: false,
            // state: SlimeMoldState::Loading,
        }
    }
}

impl render_graph::Node for OceanComputeNode {
    fn update(&mut self, _world: &mut World) {
        self.update_spectrum = true;
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let compute_pipelines = world.resource::<OceanComputePipeline>();

        let gpu_images = world.resource::<RenderAssets<Image>>();
        let ocean_textures = world.resource::<OceanComputeTextures>();

        let uniforms = world.resource::<OceanComputeUniforms>();
        let spectrums = world.resource::<OceanSpectrumStorage>();

        let displacement_textures = &gpu_images[&ocean_textures.displacements];
        let gradient_textures = &gpu_images[&ocean_textures.gradients];
        let init_spectrum_textures = &gpu_images[&ocean_textures.init_spectrum_textures];
        let spectrum_textures = &gpu_images[&ocean_textures.spectrum_textures];

        let bind_group = render_context
            .render_device()
            .create_bind_group(&BindGroupDescriptor {
                label: Some("ocean_compute_pass_bind_group"),
                layout: &compute_pipelines.layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: uniforms.buf.binding().unwrap(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: spectrums.buf.binding().unwrap(),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::TextureView(&displacement_textures.texture_view),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: BindingResource::TextureView(&gradient_textures.texture_view),
                    },
                    BindGroupEntry {
                        binding: 4,
                        resource: BindingResource::TextureView(&init_spectrum_textures.texture_view),
                    },
                    BindGroupEntry {
                        binding: 5,
                        resource: BindingResource::TextureView(&spectrum_textures.texture_view),
                    },
                ],
            });

        let encoder = render_context.command_encoder();

        if self.update_spectrum {
            {
                let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());

                pass.set_bind_group(0, &bind_group, &[]);

                let Some(pipeline) = pipeline_cache.get_compute_pipeline(compute_pipelines.init_spectrum_pipeline) else {
                    return Ok(());
                };
                pass.set_pipeline(pipeline);
                pass.dispatch_workgroups(TEXTURE_SIZE / WORKGROUP_SIZE, TEXTURE_SIZE / WORKGROUP_SIZE, 1);                
            }
            {
                let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());

                pass.set_bind_group(0, &bind_group, &[]);

                let Some(pipeline) = pipeline_cache.get_compute_pipeline(compute_pipelines.pack_spectrum_conj_pipeline) else {
                    return Ok(());
                };
                pass.set_pipeline(pipeline);
                pass.dispatch_workgroups(TEXTURE_SIZE / WORKGROUP_SIZE, TEXTURE_SIZE / WORKGROUP_SIZE, 1);                
            }
        }

        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());

            pass.set_bind_group(0, &bind_group, &[]);

            let Some(pipeline) = pipeline_cache.get_compute_pipeline(compute_pipelines.update_spectrum_pipeline) else {
                return Ok(());
            };
            pass.set_pipeline(pipeline);
            pass.dispatch_workgroups(TEXTURE_SIZE / WORKGROUP_SIZE, TEXTURE_SIZE / WORKGROUP_SIZE, 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());

            pass.set_bind_group(0, &bind_group, &[]);

            let Some(pipeline) = pipeline_cache.get_compute_pipeline(compute_pipelines.horizontal_fft_pipeline) else {
                return Ok(());
            };
            pass.set_pipeline(pipeline);
            pass.dispatch_workgroups(1, TEXTURE_SIZE, 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());

            pass.set_bind_group(0, &bind_group, &[]);

            let Some(pipeline) = pipeline_cache.get_compute_pipeline(compute_pipelines.vertical_fft_pipeline) else {
                return Ok(());
            };
            pass.set_pipeline(pipeline);
            pass.dispatch_workgroups(1, TEXTURE_SIZE, 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());

            pass.set_bind_group(0, &bind_group, &[]);

            let Some(pipeline) = pipeline_cache.get_compute_pipeline(compute_pipelines.assemble_maps_pipeline) else {
                return Ok(());
            };
            pass.set_pipeline(pipeline);
            pass.dispatch_workgroups(TEXTURE_SIZE / WORKGROUP_SIZE, TEXTURE_SIZE / WORKGROUP_SIZE, 1);
        }

        Ok(())
    }
}