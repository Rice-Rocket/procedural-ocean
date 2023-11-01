use bevy::{prelude::*, reflect::TypeUuid, render::render_resource::{AsBindGroup, ShaderType}, asset::load_internal_asset};

use crate::pass::uniforms::OceanComputeTextures;


pub const OCEAN_MATERIAL_HANDLE: HandleUntyped = 
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0x47c886145eab339e);


#[derive(AsBindGroup, Debug, Reflect, Clone, TypeUuid)]
#[reflect(Debug, Default)]
#[uuid = "a173c451-405b-48c4-ba15-cbfef5b082b5"]
pub struct OceanMaterial {
    #[uniform(0, visibility(vertex, fragment))]
    pub settings: OceanSettings,

    #[texture(1, visibility(vertex, fragment))]
    #[sampler(2, visibility(vertex, fragment))]
    pub displacements: Option<Handle<Image>>,
}

impl Material for OceanMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/ocean.wgsl".into()
    }
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/ocean.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}

impl Default for OceanMaterial {
    fn default() -> Self {
        Self {
            settings: OceanSettings::default(),
            displacements: None,
        }
    }
}


#[derive(Debug, Clone, Reflect, ShaderType)]
pub struct OceanSettings {
    pub time: f32,
    pub frequency: f32,
    pub amplitude: f32,
}

impl Default for OceanSettings {
    fn default() -> Self {
        Self {
            time: 0.0,
            frequency: 10.0,
            amplitude: 0.25,
        }
    }
}


pub fn prepare_ocean_material(
    handles: Query<&Handle<OceanMaterial>>,
    mut materials: ResMut<Assets<OceanMaterial>>,

    compute_textures: Res<OceanComputeTextures>,
    time: Res<Time>,
) {
    for handle in handles.iter() {
        let mat = materials.get_mut(handle).unwrap();

        mat.settings.time = time.elapsed_seconds();
        
        if mat.displacements.is_none() {
            mat.displacements = Some(compute_textures.displacement.clone());
        }
    }
}


pub struct OceanMaterialPlugin;

impl Plugin for OceanMaterialPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            OCEAN_MATERIAL_HANDLE,
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/ocean.wgsl"),
            Shader::from_wgsl
        );

        app
            .add_plugins(MaterialPlugin::<OceanMaterial>::default())
            .add_systems(Update, prepare_ocean_material)
            .register_type::<OceanMaterial>()
            .register_asset_reflect::<OceanMaterial>()
            .register_type::<Handle<OceanMaterial>>();
    }
}