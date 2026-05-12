use std::mem::offset_of;

use ash::vk;
use bytemuck::{Pod, Zeroable};

pub trait AttributeDescriptions {
    fn attr_desc() -> Vec<vk::VertexInputAttributeDescription>;
}

pub trait BindingDescriptions {
    fn bind_desc() -> Vec<vk::VertexInputBindingDescription>;
}

/// Basic simple vertex
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    /// Pos Vertex
    pub pos: [f32; 3],
    /// Color Vertex
    pub color: [f32; 3],
}

impl Vertex {
    /// Create cube mesh
    #[allow(dead_code)]
    pub fn cube(_x: f32, _y: f32, _z: f32) -> Vec<Vertex> {
        let vertices = vec![];

        vertices
    }

    pub fn triangle(_x: f32, _y: f32, _z: f32) -> Vec<Vertex> {
        vec![]
    }
}

/// Texture Vertex
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct TextureVertex {
    /// Pos
    pub pos: [f32; 3],
    /// UV Pos
    pub uv: [f32; 2],
}

/// PBR Vertex
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct PbrVertex {
    /// Pos
    pub pos: [f32; 4],
    /// Normal
    pub normal: [f32; 4],
    /// Uv
    pub uv: [f32; 2],
    /// Color
    pub color: [f32; 4],
    /// Tangent
    pub tangent: [f32; 4],
}

impl PbrVertex {
    /// Create new PBR Vertex
    #[allow(dead_code)]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            pos: [x, y, z, 0.0],
            normal: [0.0, 0.0, 0.0, 0.0],
            uv: [0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            tangent: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

impl AttributeDescriptions for PbrVertex {
    fn attr_desc() -> Vec<ash::vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(PbrVertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(PbrVertex, normal) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(PbrVertex, uv) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 3,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(PbrVertex, color) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 4,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(PbrVertex, tangent) as u32,
            },
        ]
    }
}

impl BindingDescriptions for PbrVertex {
    fn bind_desc() -> Vec<ash::vk::VertexInputBindingDescription> {
        vec![vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<PbrVertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }
}

impl BindingDescriptions for Vertex {
    fn bind_desc() -> Vec<ash::vk::VertexInputBindingDescription> {
        vec![vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }
}

impl AttributeDescriptions for Vertex {
    fn attr_desc() -> Vec<vk::VertexInputAttributeDescription> {
        let attributes = vec![
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: std::mem::offset_of!(Vertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: std::mem::offset_of!(Vertex, color) as u32,
            },
        ];
        attributes
    }
}

impl BindingDescriptions for TextureVertex {
    fn bind_desc() -> Vec<ash::vk::VertexInputBindingDescription> {
        vec![vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<TextureVertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }
}

impl AttributeDescriptions for TextureVertex {
    fn attr_desc() -> Vec<vk::VertexInputAttributeDescription> {
        let attributes = vec![
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: std::mem::offset_of!(TextureVertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32_SFLOAT,
                offset: std::mem::offset_of!(TextureVertex, uv) as u32,
            },
        ];
        attributes
    }
}


