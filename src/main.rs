use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use bevy_inspector_egui::quick::{AssetInspectorPlugin, ResourceInspectorPlugin, FilterQueryInspectorPlugin};

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
            FilterQueryInspectorPlugin::<With<SkyPostProcessSettings>>::default(),
        ))
        .insert_resource(Msaa::Off)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, skybox_loaded)
        .run();
}
