use bevy::{prelude::*, reflect::TypeUuid, render::render_resource::{AsBindGroup, ShaderType}, asset::load_internal_asset};


pub const OCEAN_MATERIAL_HANDLE: HandleUntyped = 
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0x47c886145eab339e);


#[derive(AsBindGroup, Debug, Reflect, Clone, TypeUuid)]
#[reflect(Debug, Default)]
#[uuid = "a173c451-405b-48c4-ba15-cbfef5b082b5"]
pub struct OceanMaterial {
    #[uniform(0)]
    pub settings: OceanSettings,
}

impl Material for OceanMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/ocean_frag.wgsl".into()
    }
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/ocean_vert.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}

impl Default for OceanMaterial {
    fn default() -> Self {
        Self {
            settings: OceanSettings::default(),
        }
    }
}


#[derive(Debug, Clone, Reflect, ShaderType)]
pub struct OceanSettings {
    pub wave_height: f32,
}

impl Default for OceanSettings {
    fn default() -> Self {
        Self {
            wave_height: 1.0,
        }
    }
}


pub struct OceanMaterialPlugin;

impl Plugin for OceanMaterialPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            OCEAN_MATERIAL_HANDLE,
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/ocean_frag.wgsl"),
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            OCEAN_MATERIAL_HANDLE,
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/ocean_vert.wgsl"),
            Shader::from_wgsl
        );

        app
            .add_plugins(MaterialPlugin::<OceanMaterial>::default())
            .register_type::<OceanMaterial>()
            .register_asset_reflect::<OceanMaterial>()
            .register_type::<Handle<OceanMaterial>>();
    }
}