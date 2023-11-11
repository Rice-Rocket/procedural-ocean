use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use bevy_inspector_egui::quick::{AssetInspectorPlugin, ResourceInspectorPlugin};

pub mod scene;
pub mod ocean;
pub mod pass;

use scene::*;
use ocean::*;
use pass::{*, uniforms::OceanComputeSettings, spectrums::OceanSpectrumsDisplayArray};


fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PanOrbitCameraPlugin,
            OceanMaterialPlugin,
            OceanComputePlugin,
            AssetInspectorPlugin::<OceanMaterial>::default(),
            ResourceInspectorPlugin::<OceanComputeSettings>::default(),
            ResourceInspectorPlugin::<OceanSpectrumsDisplayArray>::default(),
        ))
        .add_systems(Startup, setup_scene)
        .run();
}
