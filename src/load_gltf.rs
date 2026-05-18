use std::{collections::HashMap, path::Path, sync::Arc};
use ash::vk;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use gltf::image::Format;
use rapier3d::na::Point3;

use crate::{Bindless, Mesh, Texture, WorldRenderer, core::{DescriptorPool, DescriptorSetLayoutBuilder, GpuBuffer, GpuBufferBuilder, PbrVertex, VulkanResult}, render_context::RenderContext};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
pub struct Transform {
    pub model: [[f32; 4]; 4],
    pub normal: [[f32; 4]; 4],
}

#[derive(Clone)]
pub struct Material {
    pub diffuse_map: u32,
    pub normal_map: u32,
    pub metallic_roughness_map: u32,
    pub occlusion_map: u32,
}

pub struct GltfModelBuilder {
    pub gltf_image_to_bindless: HashMap<usize, u32>,
    pub fallback_diffuse: u32,
    pub fallback_metallic: u32,
    pub fallback_occlusion: u32,
    pub fallback_normal: u32,
    pub physic_vertices: Vec<(Vec<Vec3>, Vec<u32>)>,
    pub meshes: Vec<(Mesh, Material)>,
    pub transforms: Vec<Transform>,
    pub textures: Vec<Texture>,
}

pub struct GltfModel {
    pub physic_vertices: Vec<(Vec<Vec3>, Vec<u32>)>,
    pub meshes: Vec<(Mesh, Material)>,
    pub transforms: Vec<Transform>,
    pub textures: Vec<Texture>,
}

fn load_gltf_node(
    ctx: &Arc<RenderContext>,
    model: &mut GltfModelBuilder,
    node: gltf::Node<'_>,
    buffers: &[gltf::buffer::Data],
    parent_transform: Mat4,
) -> VulkanResult<()> {
    let node_transform = parent_transform * Mat4::from_cols_array_2d(&node.transform().matrix());

    for child in node.children() {
        load_gltf_node(ctx, model, child, buffers, node_transform)?;
    }

    if let Some(mesh) = node.mesh() {
        let primitives = mesh.primitives();
        for i in primitives {
            let reader = i.reader(|i| Some(&buffers[i.index()]));

            let indices: Vec<u32> = reader.read_indices().unwrap().into_u32().collect();
            let positions: Vec<_> = reader
                .read_positions()
                .unwrap()
                .map(|x| [x[0], x[1], x[2], 1.0])
                .collect();

            let tex_coords = if let Some(tex_coords) = reader.read_tex_coords(0) {
                tex_coords.into_f32().map(|x| x).collect()
            } else {
                vec![[0.0, 0.0]; positions.len()]
            };

            let tangents = if let Some(tangents) = reader.read_tangents() {
                tangents.map(|x| x).collect()
            } else {
                vec![[0.0, 0.0, 0.0, 0.0]; positions.len()]
            };

            let normals: Vec<_> = reader
                .read_normals()
                .unwrap()
                .map(|x| [x[0], x[1], x[2], 0.0])
                .collect();

            let base_color = i.material().pbr_metallic_roughness().base_color_factor();

            let colors: Vec<_> = if let Some(colors) = reader.read_colors(0) {
                colors.into_rgba_f32().map(|x| x).collect()
            } else {
                vec![base_color; positions.len()]
            };

            let mut vertices = vec![];

            for (index, pos) in positions.into_iter().enumerate() {
                vertices.push(PbrVertex {
                    pos,
                    normal: normals[index],
                    uv: tex_coords[index],
                    color: colors[index],
                    tangent: tangents[index],
                });
            }
            
            let mut vertex_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
                .size((size_of::<PbrVertex>() * vertices.len()) as u64)
                .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
                .build()?;

            vertex_buffer.upload_data(&vertices)?;
            
            let mut index_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
                .size((size_of::<u32>() * indices.len()) as u64)
                .usage(vk::BufferUsageFlags::INDEX_BUFFER)
                .build()
                .unwrap();

            index_buffer.upload_data(&indices)?;

            let material = i.material();
            let pbr = material.pbr_metallic_roughness();

            let diffuse_index = pbr.base_color_texture()
                .and_then(|t| model.gltf_image_to_bindless.get(&t.texture().source().index()).copied())
                .unwrap_or(model.fallback_diffuse);

            let metallic_roughness_index = pbr.metallic_roughness_texture()
                .and_then(|t| model.gltf_image_to_bindless.get(&t.texture().source().index()).copied())
                .unwrap_or(model.fallback_metallic);

            let occlusion_index = material.occlusion_texture()
                .and_then(|t| model.gltf_image_to_bindless.get(&t.texture().source().index()).copied())
                .unwrap_or(model.fallback_occlusion);

            let normal_index = material.normal_texture()
                .and_then(|t| model.gltf_image_to_bindless.get(&t.texture().source().index()).copied())
                .unwrap_or(model.fallback_normal);

            let model_matrix = node_transform;
            let normal_matrix = node_transform.inverse().transpose();

            model.transforms.push(Transform {
                model: model_matrix.to_cols_array_2d(),
                normal: normal_matrix.to_cols_array_2d(),
            });

            let phys_vertices = vertices.iter().map(|x| Vec3::new(x.pos[0], x.pos[1], x.pos[2])).collect::<Vec<_>>();

            let index_count = indices.len() as u32;

            model.physic_vertices.push(
                (phys_vertices, indices)
            );

            model.meshes.push((
                Mesh { 
                    vertex_buffer,
                    index_count,
                    index_buffer
                },
                Material {
                    diffuse_map: diffuse_index,
                    normal_map: normal_index,
                    metallic_roughness_map: metallic_roughness_index,
                    occlusion_map: occlusion_index,
                },
            ));
        }
    }

    Ok(())
}

pub fn load_gltf<P: AsRef<Path>>(world: &WorldRenderer, path: P) ->  std::thread::JoinHandle<VulkanResult<GltfModel>> {

    let path = path.as_ref().to_path_buf();

    let ctx = world.ctx.clone();
    let bindless = world.bindless.clone();

    let join_handle: std::thread::JoinHandle<VulkanResult<GltfModel>> = std::thread::spawn(move || {

        let (gltf, buffers, images) = match gltf::import(path) {
            Ok(result) => result,
            Err(err) => panic!("AAAA!"),
        };

        let mut textures = vec![];

        // index 0 - diffuse fallback (белый)
        let pixels = vec![255u8, 255, 255, 255];
        let mut tex = Texture::new(&ctx, vk::Format::R8G8B8A8_SRGB, 1, 1, &pixels).unwrap();
        let idx = bindless.alloc_texture(&ctx.device, tex.image_view.raw);
        tex.bindless_index = Some(idx);
        textures.push(tex); // bindless 0

        // index 1 - metallic_roughness fallback (не металл, средняя шершавость)
        let pixels = vec![0u8, 128, 0, 255]; // r=occlusion, g=roughness, b=metallic
        let mut tex = Texture::new(&ctx, vk::Format::R8G8B8A8_SRGB, 1, 1, &pixels).unwrap();
        let idx = bindless.alloc_texture(&ctx.device, tex.image_view.raw);
        tex.bindless_index = Some(idx);
        textures.push(tex); // bindless 1

        // index 2 - occlusion fallback (белый = нет затенения)
        let pixels = vec![255u8, 255, 255, 255];
        let mut tex = Texture::new(&ctx, vk::Format::R8G8B8A8_SRGB, 1, 1, &pixels).unwrap();
        let idx = bindless.alloc_texture(&ctx.device, tex.image_view.raw);
        tex.bindless_index = Some(idx);
        textures.push(tex); // bindless 2

        // index 3 - normal fallback (flat normal = 128, 128, 255)
        let pixels = vec![128u8, 128, 255, 255];
        let mut tex = Texture::new(&ctx, vk::Format::R8G8B8A8_SRGB, 1, 1, &pixels).unwrap();
        let idx = bindless.alloc_texture(&ctx.device, tex.image_view.raw);
        tex.bindless_index = Some(idx);
        textures.push(tex); // bindless 3

        let mut gltf_image_to_bindless: HashMap<usize, u32> = HashMap::new();

        for (gltf_index, mut image) in images.into_iter().enumerate() {
            let pixels = std::mem::take(&mut image.pixels);
            let (width, height) = (image.width, image.height);

            let dynamic_image = match image.format {
                Format::R8G8B8A8 => image::DynamicImage::ImageRgba8(
                    image::RgbaImage::from_raw(width, height, pixels).unwrap()
                ),
                Format::R8G8B8 => image::DynamicImage::ImageRgb8(
                    image::RgbImage::from_raw(width, height, pixels).unwrap()
                ),
                Format::R8G8 => image::DynamicImage::ImageLuma8(
                    image::GrayImage::from_raw(width, height, pixels).unwrap()
                ),
                Format::R8 => image::DynamicImage::ImageLuma8(
                    image::GrayImage::from_raw(width, height, pixels).unwrap()
                ),
                Format::R16G16B16A16 => image::DynamicImage::ImageRgba16(
                    image::ImageBuffer::from_raw(width, height, 
                        pixels.chunks(2).map(|p| u16::from_le_bytes([p[0], p[1]])).collect()
                    ).unwrap()
                ),
                Format::R16G16B16 => image::DynamicImage::ImageRgb16(
                    image::ImageBuffer::from_raw(width, height,
                        pixels.chunks(2).map(|p| u16::from_le_bytes([p[0], p[1]])).collect()
                    ).unwrap()
                ),
                Format::R32G32B32A32FLOAT => image::DynamicImage::ImageRgba32F(
                    image::ImageBuffer::from_raw(width, height,
                        pixels.chunks(4).map(|p| f32::from_le_bytes([p[0], p[1], p[2], p[3]])).collect()
                    ).unwrap()
                ),
                Format::R32G32B32FLOAT => image::DynamicImage::ImageRgb32F(
                    image::ImageBuffer::from_raw(width, height,
                        pixels.chunks(4).map(|p| f32::from_le_bytes([p[0], p[1], p[2], p[3]])).collect()
                    ).unwrap()
                ),
                _ => {
                    println!("Skip texture format: {:?}", image.format);
                    continue;
                }
            };

            let rgba8 = dynamic_image.to_rgba8();
            let pixels = rgba8.into_raw();

            let mut tex = Texture::new(&ctx, vk::Format::R8G8B8A8_SRGB, width, height, &pixels).unwrap();
            let idx = bindless.alloc_texture(&ctx.device, tex.image_view.raw); // <-- здесь, не в отдельном цикле
            tex.bindless_index = Some(idx);

            gltf_image_to_bindless.insert(gltf_index, idx);

            textures.push(tex);
        }
    

        let mut builder = GltfModelBuilder {
            physic_vertices: vec![],
            fallback_diffuse: textures[0].bindless_index.unwrap(),
            fallback_metallic: textures[1].bindless_index.unwrap(),
            fallback_occlusion: textures[2].bindless_index.unwrap(),
            fallback_normal: textures[3].bindless_index.unwrap(),
            gltf_image_to_bindless,
            meshes: vec![],
            transforms: vec![],
            textures,
        };

        for scene in gltf.scenes() {
            for node in scene.nodes() {
                load_gltf_node(&ctx, &mut builder, node, &buffers, Mat4::IDENTITY)?;
            }
        }

        Ok(GltfModel { 
            physic_vertices: builder.physic_vertices,
            meshes: builder.meshes, 
            transforms: builder.transforms, 
            textures: builder.textures
        })
    });

    join_handle
}
