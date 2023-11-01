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

use uniforms::*;

use self::{node::OceanComputeNode, pipeline::OceanComputePipeline};

pub const DEFAULT_TEXTURE_SIZE: u32 = 512;
pub const WORKGROUP_SIZE: u32 = 32;


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
            .add_systems(Startup, setup_textures)
            .add_plugins((
                ExtractResourcePlugin::<OceanComputeSettings>::default(),
                ExtractResourcePlugin::<OceanComputeTextures>::default(),
            ));

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<OceanComputeUniforms>()
            .add_state::<SimulationState>()
            .add_systems(Render, prepare_uniforms.in_set(RenderSet::Prepare));
        
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