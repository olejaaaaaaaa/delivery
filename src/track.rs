use std::{sync::Arc, time::Instant};
use ash::vk;
use glam::{Mat4, Quat, Vec3};
use rapier3d::{dynamics::{GenericJointBuilder, ImpulseJointHandle, ImpulseJointSet, JointAxesMask, JointAxis, RigidBodyBuilder, RigidBodyHandle, RigidBodySet}, geometry::{ColliderBuilder, InteractionGroups}};
use winit::{event::{KeyEvent, WindowEvent}, event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowId}};
use crate::{BackgroundUiRenderer, Bindless, Camera, EguiRenderer, GltfModel, Loader2Renderer, LoaderRenderer, Mesh, PbrRenderer, PhysicsEngine, RenderLoop, SkyRenderer, Texture, Transform, WorldRenderer, core::{DescriptorPool, DescriptorPoolBuilder, DescriptorSetLayout, DescriptorSetLayoutBuilder, GpuBuffer, GpuBufferBuilder, TextureVertex, Vertex, VulkanResult}, load_gltf, open_vfs, render_context::{GraphicsDevice, RenderContext}};

pub struct Track {
    pub set: Option<vk::DescriptorSet>,
    pub set_layout: Option<DescriptorSetLayout>,
    pub track_model_task: Option<std::thread::JoinHandle<VulkanResult<GltfModel>>>,
    pub model: Option<GltfModel>,
    pub transforms: Option<GpuBuffer>
}

impl Track {
    pub fn new(world: &WorldRenderer) -> VulkanResult<Self> {
        let model = load_gltf(world ,open_vfs(r"C:\Users\Oleja\Desktop\cornell-mcray\assets\track\track.gltf"));

        Ok(Self {
            set: None,
            set_layout: None,
            transforms: None,
            model: None,
            track_model_task: Some(model)
        })
    }

    pub fn load_if_finish(&mut self, world: &WorldRenderer, physics: &mut PhysicsEngine) {
        
        if self.track_model_task.as_ref().map_or(false, |h| h.is_finished()) {
            
            let model = self.track_model_task.take().unwrap().join().unwrap().unwrap();

            for (vertices, indices) in &model.physic_vertices {
                let indices: Vec<[u32; 3]> = indices
                    .chunks(3)
                    .map(|c| [c[0], c[1], c[2]])
                    .collect();

                let collider = ColliderBuilder::trimesh(vertices.clone(), indices).unwrap().build();
                let rigid_body= RigidBodyBuilder::fixed().build();
                let rigid_handle = physics.create_rigid_body(rigid_body);
                physics.create_collider(collider, rigid_handle);
            }
            
            let mut storage_buffer = GpuBufferBuilder::cpu_only(&world.ctx.device)
                .size((size_of::<Transform>() * model.transforms.len()) as u64)
                .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
                .build()
                .unwrap();

            storage_buffer.upload_data(&model.transforms).unwrap();

            let set_layout = DescriptorSetLayoutBuilder::new(&world.ctx.device)
                .bindings(&[
                    vk::DescriptorSetLayoutBinding::default()
                        .stage_flags(vk::ShaderStageFlags::VERTEX)
                        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                        .descriptor_count(1)
                ])
                .build()
                .unwrap();

            let set = world.resources.pool.create_descriptor_set(&world.ctx.device, &[set_layout.raw]).unwrap()[0];

            unsafe {

                let buffer_info = vk::DescriptorBufferInfo::default()   
                    .buffer(storage_buffer.raw)
                    .offset(0)
                    .range(vk::WHOLE_SIZE);

                let write = vk::WriteDescriptorSet::default()
                    .descriptor_count(1)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .dst_set(set)
                    .dst_binding(0)
                    .dst_array_element(0)
                    .buffer_info(std::slice::from_ref(&buffer_info));

                world.ctx.device.update_descriptor_sets(&[write], &[]);
            }

            self.model = Some(model);
            self.set_layout = Some(set_layout);
            self.set = Some(set);
            self.transforms = Some(storage_buffer);
        }
    }
}