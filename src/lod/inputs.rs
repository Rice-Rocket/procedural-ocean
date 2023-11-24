use bevy::{prelude::*, render::{render_resource::{ShaderType, Buffer, BufferDescriptor, BufferUsages, encase, BufferInitDescriptor}, extract_component::ExtractComponent, render_asset::RenderAssets, renderer::RenderDevice, mesh::GpuBufferInfo}, utils::HashMap};


pub const VERTEX_OUT_BUFFER_SIZE: u32 = 2u32.pow(16u32) * 3u32;
pub const INDEX_OUT_BUFFER_SIZE: u32 = 2u32.pow(16u32);


#[derive(Component, ExtractComponent, Clone, ShaderType)]
pub struct DynamicDetailMesh {
    pub max_level: u32,
}

#[derive(Component, ExtractComponent, Clone)]
pub struct DynamicDetailBufferData {
    pub subd_buf_size: u32,
}

#[derive(Resource, Default)]
pub struct DynamicDetailBuffers {
    pub subdivision_keys: HashMap<Entity, Buffer>,
    pub vertices_in: HashMap<Entity, Buffer>,
    pub indices_in: HashMap<Entity, Buffer>,
    pub vertices_out: HashMap<Entity, Buffer>,
    pub indices_out: HashMap<Entity, Buffer>,
}

pub fn init_lod_buffer_data(
    mut commands: Commands,
    lod_meshes: Query<(Entity, &Handle<Mesh>, &DynamicDetailMesh), Without<DynamicDetailBufferData>>,
    meshes: Res<Assets<Mesh>>,
) {
    for (entity, mesh_handle, settings) in lod_meshes.iter() {
        let Some(mesh) = meshes.get(mesh_handle) else { continue };

        // let subd_buf_size = 4u32.pow(settings.max_level);
        let subd_buf_size = 3 * settings.max_level + 1;

        commands.entity(entity).insert(DynamicDetailBufferData {
            subd_buf_size,
        });
    }
}

pub fn update_lod_input(
    lod_meshes: Query<(Entity, &Handle<Mesh>, &DynamicDetailMesh, &DynamicDetailBufferData)>,
    mut lod_buffers: ResMut<DynamicDetailBuffers>,
    meshes: Res<RenderAssets<Mesh>>,
    render_device: Res<RenderDevice>,
) {
    for (entity, mesh_handle, settings, buf_data) in lod_meshes.iter() {
        let Some(mesh) = meshes.get(mesh_handle) else { continue };

        if !lod_buffers.subdivision_keys.contains_key(&entity) {
            let buf = render_device.create_buffer(&BufferDescriptor {
                label: None,
                size: buf_data.subd_buf_size as u64 * std::mem::size_of::<u32>() as u64,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            lod_buffers.subdivision_keys.insert(entity, buf);
        }

        if !lod_buffers.vertices_in.contains_key(&entity) {
            let vertices = mesh.vertex_buffer;
            lod_buffers.vertices_in.insert(entity, vertices);
        }

        if !lod_buffers.indices_in.contains_key(&entity) {
            let GpuBufferInfo::Indexed { buffer: indices, count: idx_count, index_format }
                = mesh.buffer_info else { continue };
            lod_buffers.indices_in.insert(entity, indices);
        }

        if !lod_buffers.vertices_out.contains_key(&entity) {
            let buf = render_device.create_buffer(&BufferDescriptor {
                label: None,
                size: VERTEX_OUT_BUFFER_SIZE as u64 * std::mem::size_of::<f32>() as u64,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            lod_buffers.vertices_out.insert(entity, buf);
        }

        if !lod_buffers.indices_out.contains_key(&entity) {
            let buf = render_device.create_buffer(&BufferDescriptor {
                label: None,
                size: INDEX_OUT_BUFFER_SIZE as u64 * std::mem::size_of::<u32>() as u64,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            lod_buffers.indices_out.insert(entity, buf);
        }
    }
}