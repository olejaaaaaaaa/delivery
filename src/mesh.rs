use crate::core::GpuBuffer;

pub struct Mesh {
    pub vertex_buffer: GpuBuffer,
    pub index_buffer: GpuBuffer,
    pub index_count: u32
}