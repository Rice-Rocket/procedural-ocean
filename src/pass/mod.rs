use bevy::{
    prelude::*, 
    render::{
        extract_resource::ExtractResourcePlugin, RenderApp, Render, 
        render_graph::RenderGraph, 
        RenderSet
    }, 
};

pub mod uniforms;
pub mod pipeline;
pub mod node;
pub mod spectrums;

use uniforms::*;
use spectrums::*;

use self::{node::OceanComputeNode, pipeline::OceanComputePipeline, spectrums::{OceanSpectrumsArray, OceanSpectrumStorage}};

// Should match with workgroup size of horizontal fft and vertical fft and SIZE constant in compute shader
pub const TEXTURE_SIZE: u32 = 256;
pub const WORKGROUP_SIZE: u32 = 8;


#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
pub enum SimulationState {
    #[default]
    Uninitialized,
    Started,
}


pub struct OceanComputePlugin;

impl Plugin for OceanComputePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<OceanComputeSettings>()
            .init_resource::<OceanSpectrumsArray>()
            .init_resource::<OceanSpectrumsDisplayArray>()
            .add_systems(Startup, setup_textures)
            .add_plugins((
                ExtractResourcePlugin::<OceanComputeSettings>::default(),
                ExtractResourcePlugin::<OceanSpectrumsArray>::default(),
                ExtractResourcePlugin::<OceanSpectrumsDisplayArray>::default(),
                ExtractResourcePlugin::<OceanComputeTextures>::default(),
            ));

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<OceanComputeUniforms>()
            .init_resource::<OceanSpectrumStorage>()
            .add_state::<SimulationState>()
            .add_systems(Render, (prepare_uniforms, prepare_storage).in_set(RenderSet::Prepare));
        
        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node(OceanComputeNode::NAME, OceanComputeNode::default());
        render_graph.add_node_edges(&[
            OceanComputeNode::NAME,
            bevy::render::main_graph::node::CAMERA_DRIVER,
        ]);
    }
    
    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<OceanComputePipeline>();
    }
}