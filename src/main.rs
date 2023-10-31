use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCameraPlugin;

pub mod scene;
pub mod ocean;

use scene::*;
use ocean::*;


fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PanOrbitCameraPlugin,
            OceanMaterialPlugin,
        ))
        .add_systems(Startup, setup_scene)
        .run();
}
