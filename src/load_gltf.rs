use std::{path::Path, sync::Arc};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use gltf::image::Format;

use crate::{Mesh, Texture, core::{GpuBuffer, PbrVertex, VulkanResult}, render_context::RenderContext};

// #[repr(C)]
// #[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
// pub struct Transform {
//     pub model: [[f32; 4]; 4],
//     pub view: [[f32; 4]; 4],
//     pub proj: [[f32; 4]; 4],
//     pub normal: [[f32; 4]; 4],
// }

// #[derive(Clone)]
// pub struct Material {
//     pub diffuse_map: u32,
//     pub normal_map: u32,
//     pub metallic_roughness_map: u32,
//     pub occlusion_map: u32,
// }

// pub struct GltfModel {
//     pub meshes: Vec<(Mesh, Material)>,
//     pub transforms: Vec<Transform>,
//     pub ssbo: GpuBuffer,
//     pub textures: Vec<Texture>,
// }

// fn load_gltf_node(
//     world: &Arc<RenderContext>,
//     model: &mut GltfModel,
//     node: gltf::Node<'_>,
//     buffers: &[gltf::buffer::Data],
//     parent_transform: Mat4,
// ) -> VulkanResult<()> {
//     let node_transform = parent_transform * Mat4::from_cols_array_2d(&node.transform().matrix());

//     for child in node.children() {
//         load_gltf_node(world, model, child, buffers, node_transform)?;
//     }

//     if let Some(mesh) = node.mesh() {
//         let primitives = mesh.primitives();
//         for i in primitives {
//             let reader = i.reader(|i| Some(&buffers[i.index()]));

//             let indices: Vec<u32> = reader.read_indices().unwrap().into_u32().collect();
//             let positions: Vec<_> = reader
//                 .read_positions()
//                 .unwrap()
//                 .map(|x| [x[0], x[1], x[2], 1.0])
//                 .collect();

//             let tex_coords = if let Some(tex_coords) = reader.read_tex_coords(0) {
//                 tex_coords.into_f32().map(|x| x).collect()
//             } else {
//                 vec![[0.0, 0.0]; positions.len()]
//             };

//             let tangents = if let Some(tangents) = reader.read_tangents() {
//                 tangents.map(|x| x).collect()
//             } else {
//                 vec![[0.0, 0.0, 0.0, 0.0]; positions.len()]
//             };

//             let normals: Vec<_> = reader
//                 .read_normals()
//                 .unwrap()
//                 .map(|x| [x[0], x[1], x[2], 0.0])
//                 .collect();

//             let colors: Vec<_> = if let Some(colors) = reader.read_colors(0) {
//                 colors.into_rgba_f32().map(|x| x).collect()
//             } else {
//                 vec![[0.9, 0.6, 0.4, 1.0]; positions.len()]
//             };

//             let mut vertices = vec![];

//             for (index, pos) in positions.into_iter().enumerate() {
//                 vertices.push(PbrVertex {
//                     pos,
//                     normal: normals[index],
//                     uv: tex_coords[index],
//                     color: colors[index],
//                     tangent: tangents[index],
//                 });
//             }

//             let vertex = world.create::<VertexBuffer>(VertexBufferDesc::new(&vertices))?;
//             let index = world.create::<IndexBuffer>(IndexBufferDesc::new(&indices))?;

//             let proj = Mat4::perspective_rh(45.0_f32.to_radians(), 800.0 / 600.0, 0.1, 1000.0);
//             let view = Mat4::look_at_rh(
//                 Vec3::new(0.0, 0.0, 1.0),
//                 Vec3::new(0.0, -0.12, 0.0),
//                 Vec3::NEG_Y,
//             );

//             let material = i.material();
//             let pbr = material.pbr_metallic_roughness();

//             let diffuse_index = pbr
//                 .base_color_texture()
//                 .map_or(0, |texture| texture.texture().index() as u32);

//             let normal_index = material
//                 .normal_texture()
//                 .map_or(0, |texture| texture.texture().index() as u32);

//             let metallic_roughness_index = pbr
//                 .metallic_roughness_texture()
//                 .map_or(0, |texture| texture.texture().index() as u32);

//             let occlusion_index = material
//                 .occlusion_texture()
//                 .map_or(0, |texture| texture.texture().index() as u32);

//             let model_matrix = node_transform;
//             let view_matrix = view;
//             let proj_matrix = proj;
//             let normal_matrix = node_transform.inverse().transpose();

//             model.transforms.push(Transform {
//                 model: model_matrix.to_cols_array_2d(),
//                 view: view_matrix.to_cols_array_2d(),
//                 proj: proj_matrix.to_cols_array_2d(),
//                 normal: normal_matrix.to_cols_array_2d(),
//             });

//             model.meshes.push((
//                 Mesh { vertex, index },
//                 Material {
//                     diffuse_map: diffuse_index,
//                     normal_map: normal_index,
//                     metallic_roughness_map: metallic_roughness_index,
//                     occlusion_map: occlusion_index,
//                 },
//             ));
//         }
//     }

//     Ok(())
// }

// pub fn load_gltf<P: AsRef<Path>>(ctx: &Arc<RenderContext>, path: P) -> VulkanResult<GltfModel> {
//     let (gltf, buffers, images) = match gltf::import(path.as_ref()) {
//         Ok(result) => result,
//         Err(err) => panic!(
//             "Error load gltf model with path: {:?} with error: {:?}",
//             path.as_ref().as_os_str(),
//             err
//         ),
//     };

//     let mut textures = vec![];

//     for mut image in images {
//         if image.format == Format::R8G8B8A8 {
//             let dynamic_image = image::DynamicImage::ImageRgba8(
//                 image::RgbaImage::from_raw(
//                     image.width,
//                     image.height,
//                     std::mem::take(&mut image.pixels),
//                 )
//                 .unwrap(),
//             );

//             let rgba8_image = dynamic_image.to_rgba8();
//             let texture = world.create::<aluminium::Texture>(TextureDesc {
//                 width: image.width,
//                 height: image.height,
//                 format: aluminium::PixelFormat::Rgba8,
//                 pixels: &rgba8_image.into_raw(),
//             })?;

//             textures.push(texture);
//         } else if image.format == Format::R8G8B8 {
//             let dynamic_image = image::DynamicImage::ImageRgb8(
//                 image::RgbImage::from_raw(
//                     image.width,
//                     image.height,
//                     std::mem::take(&mut image.pixels),
//                 )
//                 .unwrap(),
//             );

//             let rgba8_image = dynamic_image.to_rgba8();
//             let texture = world.create::<aluminium::Texture>(TextureDesc {
//                 width: image.width,
//                 height: image.height,
//                 format: aluminium::PixelFormat::Rgb8,
//                 pixels: &rgba8_image.into_raw(),
//             })?;

//             textures.push(texture);
//         } else {
//             println!("Skip texture foramt: {:?}", image.format);
//         }
//     }

//     let mut gltf_model = GltfModel {
//         meshes: vec![],
//         ssbo: None,
//         transforms: vec![],
//         textures,
//     };

//     for scene in gltf.scenes() {
//         for node in scene.nodes() {
//             load_gltf_node(world, &mut gltf_model, node, &buffers, Mat4::IDENTITY)?;
//         }
//     }

//     let transforms =
//         world.create::<StorageBuffer>(StorageBufferDesc::new(&gltf_model.transforms))?;
//     gltf_model.ssbo = Some(transforms);

//     Ok(gltf_model)
// }
