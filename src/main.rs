use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use bevy_inspector_egui::quick::{AssetInspectorPlugin, ResourceInspectorPlugin};

pub mod scene;
pub mod ocean;
pub mod compute;
pub mod sky;

use scene::*;
use ocean::*;
use compute::{*, uniforms::OceanComputeSettings, spectrums::OceanSpectrumsDisplayArray};
use sky::*;


fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PanOrbitCameraPlugin,
            OceanMaterialPlugin,
            OceanComputePlugin,
            SkyPostProcessPlugin,
            AssetInspectorPlugin::<OceanMaterial>::default(),
            ResourceInspectorPlugin::<OceanComputeSettings>::default(),
            ResourceInspectorPlugin::<OceanSpectrumsDisplayArray>::default(),
        ))
        .insert_resource(Msaa::Off)
        .add_systems(Startup, setup_scene)
        .run();
}
