use bevy::{prelude::*, reflect::TypeUuid, render::render_resource::{AsBindGroup, ShaderType}, asset::load_internal_asset};

use crate::{compute::uniforms::OceanComputeTextures, scene::SkyboxCubemap, sky::SkyPostProcessSettings};


pub const OCEAN_MATERIAL_HANDLE: HandleUntyped = 
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0x47c886145eab339e);


#[derive(AsBindGroup, Debug, Reflect, Clone, TypeUuid)]
#[reflect(Debug, Default)]
#[uuid = "a173c451-405b-48c4-ba15-cbfef5b082b5"]
pub struct OceanMaterial {
    #[uniform(0, visibility(vertex, fragment))]
    pub settings: OceanSettings,
    #[uniform(1)]
    pub sky_settings: SkyPostProcessSettings,

    #[texture(2, visibility(vertex, fragment), dimension = "2d_array")]
    #[sampler(3)]
    pub displacements: Option<Handle<Image>>,
    #[texture(4, visibility(vertex, fragment), dimension = "2d_array")]
    #[sampler(5)]
    pub gradients: Option<Handle<Image>>,

    #[texture(6, dimension = "cube")]
    #[sampler(7)]
    pub skybox: Option<Handle<Image>>,
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
            sky_settings: SkyPostProcessSettings::default(),
            displacements: None,
            gradients: None,
            skybox: None,
        }
    }
}


#[derive(Debug, Clone, Reflect, ShaderType)]
pub struct OceanSettings {
    pub base_color: Vec3,
    pub displacement_depth_attenuation: f32,
    pub low_scatter: Vec3,
    pub normal_strength: f32,
    pub sea_water_color: Vec3,
    pub roughness: f32,
    pub sun_power: f32,
    pub ocean_depth: f32,
    pub subsurface_strength: f32,

    pub tile_layers: Vec4,
    pub contribute_layers: Vec4,
}

impl Default for OceanSettings {
    fn default() -> Self {
        Self {
            base_color: Vec3::new(0.1, 0.21, 0.35),
            displacement_depth_attenuation: 1.0,
            low_scatter: Vec3::new(1.0, 0.7, 0.5),
            normal_strength: 0.5,
            sea_water_color: Vec3::new(0.8, 0.9, 0.6) * 0.6,
            roughness: 0.3,
            sun_power: 50.0,
            ocean_depth: 0.6,
            subsurface_strength: 0.6,

            tile_layers: Vec4::new(4.0, 8.0, 64.0, 128.0),
            contribute_layers: Vec4::new(1.0, 1.0, 1.0, 0.0),
        }
    }
}


pub fn prepare_ocean_material(
    handles: Query<&Handle<OceanMaterial>>,
    mut materials: ResMut<Assets<OceanMaterial>>,
    skybox: Res<SkyboxCubemap>,

    compute_textures: Res<OceanComputeTextures>,
) {
    for handle in handles.iter() {
        let mat = materials.get_mut(handle).unwrap();

        if mat.displacements.is_none() {
            mat.displacements = Some(compute_textures.displacements.clone());
            mat.gradients = Some(compute_textures.gradients.clone());
        }

        if mat.skybox.is_none() && skybox.is_loaded {
            mat.skybox = Some(skybox.skybox.clone());
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