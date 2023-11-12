use bevy::{prelude::*, render::{render_resource::PrimitiveTopology, mesh::Indices}, core_pipeline::clear_color::ClearColorConfig};
use bevy_panorbit_camera::PanOrbitCamera;

use crate::{ocean::OceanMaterial, sky::SkyPostProcessSettings};

pub const PLANE_LENGTH: f32 = 200.0;
pub const PLANE_RES: usize = 2;
// pub const PLANE_LENGTH: f32 = 10.0;
// pub const PLANE_RES: usize = 10;


pub fn setup_scene(
    mut commands: Commands,
    mut materials: ResMut<Assets<OceanMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        Camera3dBundle {
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            transform: Transform::from_xyz(4.0, 2.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
        SkyPostProcessSettings::default(),
    ));

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(create_ocean_plane()),
        material: materials.add(OceanMaterial::default()),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 20_000.0,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, 0.0, -0.1, 0.0)),
        ..default()
    });
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