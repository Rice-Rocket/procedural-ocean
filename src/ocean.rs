use bevy::{prelude::*, reflect::TypeUuid, render::render_resource::{AsBindGroup, ShaderType}, asset::load_internal_asset};

use crate::{compute::uniforms::OceanComputeTextures, sky::{SkyPostProcessSettings, SkyboxCubemap}};


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
    fn prepass_fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/prepass.wgsl".into()
    }
    fn prepass_vertex_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/prepass.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
    fn specialize(
            _pipeline: &bevy::pbr::MaterialPipeline<Self>,
            descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
            _layout: &bevy::render::mesh::MeshVertexBufferLayout,
            _key: bevy::pbr::MaterialPipelineKey<Self>,
        ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor.vertex.shader_defs.push("DEPTH_CLAMP_ORTHO".into());
        if let Some(fragment) = descriptor.fragment.as_mut() {
            fragment.shader_defs.push("DEPTH_CLAMP_ORTHO".into());
        }
        Ok(())
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
    pub normal_strength: f32,
    pub specular_normal_strength: f32,

    pub roughness: f32,
    pub foam_roughness: f32,

    pub sun_power: f32,
    pub scatter_color: Vec3,
    pub bubble_color: Vec3,
    pub foam_color: Vec3,
    pub height_modifier: f32,
    pub bubble_density: f32,

    pub wave_peak_scatter_strength: f32,
    pub scatter_strength: f32,
    pub scatter_shadow_strength: f32,
    pub environment_light_strength: f32,

    pub foam_subtract: f32,
    
    pub tile_layers: Vec4,
    pub contribute_layers: Vec4,
}

impl Default for OceanSettings {
    fn default() -> Self {
        Self {
            normal_strength: 2.0,
            specular_normal_strength: 3.0,

            roughness: 0.075,
            foam_roughness: 1.0,

            sun_power: 5.0,
            bubble_color: Vec3::new(0.01, 0.07, 0.2),
            scatter_color: Vec3::new(0.0, 0.03, 0.02),
            foam_color: Vec3::new(1.0, 1.0, 1.0),
            height_modifier: 1.5,
            bubble_density: 0.25,

            wave_peak_scatter_strength: 2.0,
            scatter_strength: 1.0,
            scatter_shadow_strength: 0.5,
            environment_light_strength: 0.4,

            foam_subtract: -0.84,

            tile_layers: Vec4::new(4.0, 8.0, 64.0, 448.0),
            contribute_layers: Vec4::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}


pub fn prepare_ocean_material(
    handles: Query<&Handle<OceanMaterial>>,
    mut materials: ResMut<Assets<OceanMaterial>>,
    skybox: Res<SkyboxCubemap>,
    sky_settings: Query<&SkyPostProcessSettings>,

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

        if let Ok(sky_setting) = sky_settings.get_single() {
            mat.sky_settings = *sky_setting;
        } else if let Some(sky_setting) = sky_settings.iter().nth(0) {
            mat.sky_settings = *sky_setting;
            warn!("More than one camera with SkyPostProcessSettings, using first in query");
        } else {
            warn!("No camera with SkyPostProcessSettings");
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