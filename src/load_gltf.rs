use std::{path::Path, sync::Arc};
use ash::vk;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use gltf::image::Format;

use crate::{Bindless, Mesh, Texture, core::{GpuBuffer, GpuBufferBuilder, PbrVertex, VulkanResult}, render_context::RenderContext};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
pub struct Transform {
    pub model: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
    pub normal: [[f32; 4]; 4],
}


#[derive(Clone)]
pub struct Material {
    pub diffuse_map: u32,
    pub normal_map: u32,
    pub metallic_roughness_map: u32,
    pub occlusion_map: u32,
}

pub struct GltfModel {
    pub meshes: Vec<(Mesh, Material)>,
    pub transforms: Vec<Transform>,
    pub ssbo: Option<GpuBuffer>,
    pub textures: Vec<Texture>,
}

fn load_gltf_node(
    ctx: &Arc<RenderContext>,
    model: &mut GltfModel,
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

            let colors: Vec<_> = if let Some(colors) = reader.read_colors(0) {
                colors.into_rgba_f32().map(|x| x).collect()
            } else {
                vec![[0.9, 0.6, 0.4, 1.0]; positions.len()]
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

            let proj = Mat4::perspective_rh(45.0_f32.to_radians(), 800.0 / 600.0, 0.1, 1000.0);
            let view = Mat4::look_at_rh(
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(0.0, -0.12, 0.0),
                Vec3::NEG_Y,
            );

            let material = i.material();
            let pbr = material.pbr_metallic_roughness();

            let diffuse_index  = pbr.base_color_texture()
                .map_or(0, |t| model.textures[t.texture().source().index() + 4].bindless_index.unwrap());

            let metallic_roughness_index = pbr.metallic_roughness_texture()
                .map_or(1, |t| model.textures[t.texture().source().index() + 4].bindless_index.unwrap());

            let occlusion_index = material.occlusion_texture()
                .map_or(2, |t| model.textures[t.texture().source().index() + 4].bindless_index.unwrap());

            let normal_index = material.normal_texture()
                .map_or(3, |t| model.textures[t.texture().source().index() + 4].bindless_index.unwrap());

            let model_matrix = node_transform;
            let view_matrix = view;
            let proj_matrix = proj;
            let normal_matrix = node_transform.inverse().transpose();

            model.transforms.push(Transform {
                model: model_matrix.to_cols_array_2d(),
                view: view_matrix.to_cols_array_2d(),
                proj: proj_matrix.to_cols_array_2d(),
                normal: normal_matrix.to_cols_array_2d(),
            });

            model.meshes.push((
                Mesh { 
                    vertex_buffer,
                    index_count: indices.len() as u32,
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

pub fn load_gltf<P: AsRef<Path>>(ctx: &Arc<RenderContext>, bindless: &Bindless, path: P) -> VulkanResult<GltfModel> {
    let (gltf, buffers, images) = match gltf::import(path.as_ref()) {
        Ok(result) => result,
        Err(err) => panic!(
            "Error load gltf model with path: {:?} with error: {:?}",
            path.as_ref().as_os_str(),
            err
        ),
    };

    let mut textures = vec![];

    // index 0 - diffuse fallback (белый)
    let pixels = vec![255u8, 255, 255, 255];
    let mut tex = Texture::new(ctx, vk::Format::R8G8B8A8_SRGB, 1, 1, &pixels)?;
    let idx = bindless.alloc_texture(&ctx.device, tex.image_view.raw);
    tex.bindless_index = Some(idx);
    textures.push(tex); // bindless 0

    // index 1 - metallic_roughness fallback (не металл, средняя шершавость)
    let pixels = vec![0u8, 128, 0, 255]; // r=occlusion, g=roughness, b=metallic
    let mut tex = Texture::new(ctx, vk::Format::R8G8B8A8_SRGB, 1, 1, &pixels)?;
    let idx = bindless.alloc_texture(&ctx.device, tex.image_view.raw);
    tex.bindless_index = Some(idx);
    textures.push(tex); // bindless 1

    // index 2 - occlusion fallback (белый = нет затенения)
    let pixels = vec![255u8, 255, 255, 255];
    let mut tex = Texture::new(ctx, vk::Format::R8G8B8A8_SRGB, 1, 1, &pixels)?;
    let idx = bindless.alloc_texture(&ctx.device, tex.image_view.raw);
    tex.bindless_index = Some(idx);
    textures.push(tex); // bindless 2

    // index 3 - normal fallback (flat normal = 128, 128, 255)
    let pixels = vec![128u8, 128, 255, 255];
    let mut tex = Texture::new(ctx, vk::Format::R8G8B8A8_SRGB, 1, 1, &pixels)?;
    let idx = bindless.alloc_texture(&ctx.device, tex.image_view.raw);
    tex.bindless_index = Some(idx);
    textures.push(tex); // bindless 3

    for mut image in images {
        let pixels = std::mem::take(&mut image.pixels);
        let (width, height) = (image.width, image.height);

        let dynamic_image = match image.format {
            Format::R8G8B8A8 => image::DynamicImage::ImageRgba8(
                image::RgbaImage::from_raw(width, height, pixels).unwrap()
            ),
            Format::R8G8B8 => image::DynamicImage::ImageRgb8(
                image::RgbImage::from_raw(width, height, pixels).unwrap()
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

        let mut tex = Texture::new(&ctx, vk::Format::R8G8B8A8_SRGB, width, height, &pixels)?;
        let idx = bindless.alloc_texture(&ctx.device, tex.image_view.raw); // <-- здесь, не в отдельном цикле
        tex.bindless_index = Some(idx);
        textures.push(tex);
    }

    println!("Texture count: {}", textures.len());

    let mut gltf_model = GltfModel {
        meshes: vec![],
        ssbo: None,
        transforms: vec![],
        textures,
    };

    for scene in gltf.scenes() {
        for node in scene.nodes() {
            load_gltf_node(ctx, &mut gltf_model, node, &buffers, Mat4::IDENTITY)?;
        }
    }

    let mut storage_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
        .size((size_of::<Transform>() * gltf_model.transforms.len()) as u64)
        .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
        .build()?;

    storage_buffer.upload_data(&gltf_model.transforms)?;
    gltf_model.ssbo = Some(storage_buffer);
    gltf_model.transforms.clear();

  

    Ok(gltf_model)
}
