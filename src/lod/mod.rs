use bevy::{prelude::*, render::extract_component::{ExtractComponentPlugin, UniformComponentPlugin}};

pub mod inputs;


pub struct OceanComputePlugin;

impl Plugin for OceanComputePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                ExtractComponentPlugin::<DynamicDetailMesh>::default(),
                UniformComponentPlugin::<DynamicDetailMesh>::default(),
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

