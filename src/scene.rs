use bevy::{prelude::*, render::{render_resource::{PrimitiveTopology, TextureViewDescriptor, TextureViewDimension}, mesh::Indices}, core_pipeline::{clear_color::ClearColorConfig, Skybox, prepass::DepthPrepass}, asset::LoadState};
use bevy_panorbit_camera::PanOrbitCamera;

use crate::{ocean::OceanMaterial, sky::{SkyPostProcessSettings, SkyboxCubemap}};

pub const PLANE_LENGTH: f32 = 200.0;
pub const PLANE_RES: usize = 4;
// pub const PLANE_LENGTH: f32 = 10.0;
// pub const PLANE_RES: usize = 10;


pub fn setup_scene(
    mut commands: Commands,
    mut materials: ResMut<Assets<OceanMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    let skybox_handle = asset_server.load("cubemaps/pure-blue.png");

    commands.spawn((
        Camera3dBundle {
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            transform: Transform::from_xyz(-10.0, 5.0, -100.0)
                .with_rotation(Quat::from_euler(EulerRot::YXZ, -std::f32::consts::PI, 0.0, std::f32::consts::PI)),
            ..default()
        },
        PanOrbitCamera::default(),
        SkyPostProcessSettings::default(),
        DepthPrepass,
    ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(create_ocean_plane()),
            material: materials.add(OceanMaterial::default()),
            // material: materials.add(OceanMaterial::default()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        // DynamicDetail {

        // },
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 20_000.0,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, 0.0, -0.1, 0.0)),
        ..default()
    });

    commands.insert_resource(SkyboxCubemap {
        skybox: skybox_handle,
        is_loaded: false,
    });
}

pub fn skybox_loaded(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut cubemap: ResMut<SkyboxCubemap>,
    mut skyboxes: Query<&mut Skybox>,
) {
    if !cubemap.is_loaded && asset_server.get_load_state(&cubemap.skybox) == LoadState::Loaded {
        let image = images.get_mut(&cubemap.skybox).unwrap();
        if image.texture_descriptor.array_layer_count() == 1 {
            image.reinterpret_stacked_2d_as_array((image.size().y / image.size().x) as u32);
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
        }
        for mut skybox in &mut skyboxes {
            skybox.0 = cubemap.skybox.clone();
        }
        cubemap.is_loaded = true;
    }
}

pub fn create_ocean_plane() -> Mesh {
    let half_length = PLANE_LENGTH * 0.5;
    let side_vert_count = PLANE_LENGTH as usize * PLANE_RES;

    let vertex_count = (side_vert_count + 1) * (side_vert_count + 1);
    let mut positions = vec![Vec3::ZERO; vertex_count];
    let mut uvs = vec![Vec2::ZERO; vertex_count];
    let mut tangents = vec![Vec4::ZERO; vertex_count];
    let tangent = Vec4::new(1.0, 0.0, 0.0, -1.0);

    let mut i = 0usize;
    for x in 0..=side_vert_count {
        for z in 0..=side_vert_count {
            positions[i] = Vec3::new(
                (x as f32 / side_vert_count as f32 * PLANE_LENGTH) - half_length,
                0.0,
                (z as f32 / side_vert_count as f32 * PLANE_LENGTH) - half_length,
            );
            uvs[i] = Vec2::new(x as f32 / side_vert_count as f32, z as f32 / side_vert_count as f32);
            tangents[i] = tangent;
            i += 1;
        }
    }

    let mut indices = vec![0u32; side_vert_count * side_vert_count * 6];

    let mut ti = 0usize;
    let mut vi = 0u32;
    for _x in 0..side_vert_count {
        for _z in 0..side_vert_count {
            indices[ti + 0] = vi;
            indices[ti + 1] = vi + 1;
            indices[ti + 2] = vi + side_vert_count as u32 + 2;
            indices[ti + 3] = vi;
            indices[ti + 4] = vi + side_vert_count as u32 + 2;
            indices[ti + 5] = vi + side_vert_count as u32 + 1;
            ti += 6;
            vi += 1;
        }
        vi += 1;
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, tangents);
    mesh.duplicate_vertices();
    mesh.compute_flat_normals();
    mesh
}