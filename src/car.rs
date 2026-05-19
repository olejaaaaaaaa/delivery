use std::{sync::Arc, thread::JoinHandle, time::Instant};
use ash::vk;
use glam::{Mat4, Quat, Vec3};
use rapier3d::{dynamics::{GenericJointBuilder, ImpulseJointHandle, ImpulseJointSet, JointAxesMask, JointAxis, RigidBodyBuilder, RigidBodyHandle, RigidBodySet}, geometry::{ColliderBuilder, ColliderHandle, InteractionGroups}};
use winit::{event::{KeyEvent, WindowEvent}, event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowId}};
use crate::{BackgroundUiRenderer, Bindless, Camera, EguiRenderer, GltfModel, Loader2Renderer, LoaderRenderer, Mesh, PbrRenderer, PhysicsEngine, RenderLoop, SkyRenderer, Texture, Transform, WorldRenderer, core::{DescriptorPool, DescriptorPoolBuilder, DescriptorSetLayout, DescriptorSetLayoutBuilder, GpuBuffer, GpuBufferBuilder, TextureVertex, Vertex, VulkanResult}, load_gltf, open_vfs, render_context::GraphicsDevice};

use super::render_context::RenderContext;


pub struct Body {

}

pub struct Wheel {

}

pub struct Car {
    pub frame_in_flight: usize,
    pub last_image: u32,
    pub car_set: Option<Vec<vk::DescriptorSet>>,
    pub car_body_task: Option<JoinHandle<VulkanResult<GltfModel>>>,
    pub car_body: Option<GltfModel>,
    pub car_body_colliders: Vec<ColliderHandle>,
    pub body_handle: Option<RigidBodyHandle>,
    pub buffer: Option<Vec<GpuBuffer>>,
    //left_wheel: GltfModel,
    //right_wheel: GltfModel,
    pub center: Vec3,
    pub position: Vec3,
    pub rotation: Quat,
    // pub body_handle: RigidBodyHandle,
    // wheel_handles: [RigidBodyHandle; 4],
    // motor_joints: Vec<ImpulseJointHandle>,
    // steering_joints: Vec<ImpulseJointHandle>,
    max_steering_angle: f32,
}

impl Car {

    pub fn new(world: &WorldRenderer, physics: &mut PhysicsEngine) -> VulkanResult<Self> {

        let car = load_gltf(world, open_vfs(r"assets\meshes\car2.glb"));
        
        // let left_up_wheel = load_gltf(world,  open_vfs(r"assets\meshes\lu_wheel.glb")).join().unwrap()?;
        // let right_up_wheel = load_gltf(world, open_vfs(r"assets\meshes\ru_wheel.glb")).join().unwrap()?;

        // let left_down_wheel = load_gltf(world,  open_vfs(r"assets\meshes\ld_wheel.glb")).join().unwrap()?;
        // let right_down_wheel = load_gltf(world, open_vfs(r"assets\meshes\rd_wheel.glb")).join().unwrap()?;

        // println!("Car body: {}", car.physic_vertices.len());
        // println!("Car Left Up wheel: {}", left_up_wheel.physic_vertices.len());
        // println!("Car Right Up wheel: {}", right_up_wheel.physic_vertices.len());
        // println!("Car Left Down wheel: {}", left_down_wheel.physic_vertices.len());
        // println!("Car Right Down wheel: {}", right_down_wheel.physic_vertices.len());

       
            

    
        // let mut car_body_colliders = vec![];

        // for (vertices, indices) in &car.physic_vertices {

        //     let indices: Vec<[u32; 3]> = indices
        //         .chunks(3)
        //         .map(|c| [c[0], c[1], c[2]])
        //         .collect();

        //     let collider = ColliderBuilder::trimesh(vertices.clone(), indices).unwrap().build();
        //     let rigid_body= RigidBodyBuilder::dynamic().build();
        //     let rigid_handle = physics.create_rigid_body(rigid_body);
        //     let collider_handle = physics.create_collider(collider, rigid_handle);

        //     car_body_colliders.push(collider_handle);
        // }

        let car_position = Vec3::new(0.0, 1.0, 0.0);

        let max_steering_angle = 35.0f32.to_radians();
        let suspension_height = 0.12;

        // // --- Кузов ---
        // let body_rb = RigidBodyBuilder::dynamic()
        //     .translation(car_position + Vec3::new(0.0, 0.47, 0.0))
        //     .build();

        // let body_handle = physics.create_rigid_body(body_rb);

        // let body_col = ColliderBuilder::cuboid(0.65, 0.3, 0.9)
        //     .friction(0.8)
        //     .build();
        
        // physics.create_collider(body_col, body_handle);

        // // --- Колёса ---
        // // (mesh_is_left, local_position_in_car_space, is_front)
        // let wheel_params = [
        //     (Vec3::new(0.6874, 0.2783, -0.7802), false), // rear left
        //     (Vec3::new(-0.6874, 0.2783, -0.7802), false), // rear right
        //     (Vec3::new( 0.64,   0.2783,  1.0254), true),  // front left
        //     (Vec3::new(-0.64,   0.2783,  1.0254), true),  // front right
        // ];

        // let mut wheel_handles = [RigidBodyHandle::invalid(); 4];
        // let mut motor_joints = vec![];
        // let mut steering_joints = vec![];

        // for (i, (local_pos, is_front)) in wheel_params.into_iter().enumerate() {
        //     let world_pos = car_position + local_pos;

        //     let wheel_rb = RigidBodyBuilder::dynamic()
        //         .translation(world_pos)
        //         .build();

        //     let wheel_handle = physics.create_rigid_body(wheel_rb);

        //     let wheel_col = ColliderBuilder::ball(0.28)
        //         .friction(1.2)
        //         .collision_groups(
        //             InteractionGroups::new(
        //                 0x02.into(), 
        //                 0x01.into(),
        //             rapier3d::geometry::InteractionTestMode::And
        //             )
        //         )
        //         .build();

        //     physics.create_collider(wheel_col, wheel_handle);

        //     wheel_handles[i] = wheel_handle;

        //     let attachment = local_pos - Vec3::new(0.0, 0.47, 0.0);

        //     let mut joint = if is_front {
        //         GenericJointBuilder::new(JointAxesMask::ANG_X | JointAxesMask::ANG_Z)
        //             .local_anchor1(attachment)
        //             .local_anchor2(Vec3::ZERO)
        //             .set_motor(JointAxis::LinY, 0.0, 0.0, 1e5, 1e4)
        //             .limits(JointAxis::LinY, [-suspension_height, suspension_height])
        //             .build()
        //     } else {
        //         GenericJointBuilder::new(
        //             JointAxesMask::ANG_X | JointAxesMask::ANG_Y | JointAxesMask::ANG_Z
        //         )
        //             .local_anchor1(attachment)
        //             .local_anchor2(Vec3::ZERO)
        //             .set_motor(JointAxis::LinY, 0.0, 0.0, 1e5, 1e4)
        //             .limits(JointAxis::LinY, [-suspension_height, suspension_height])
        //             .build()
        //     };

        //     // Мотор вращения колеса (задние)
        //     if !is_front {
        //         joint.set_motor(JointAxis::AngX, 0.0, 0.0, 0.0, 100.0);
        //         let jh = physics.create_joint(body_handle, wheel_handle, joint, true);
        //         motor_joints.push(jh);
        //     } else {
        //         joint.set_motor(JointAxis::AngY, 0.0, 0.0, 1e5, 1e4);
        //         let jh = physics.create_joint(body_handle, wheel_handle, joint, true);
        //         steering_joints.push(jh);
        //     }
        // }

        Ok(Self {
            center: Vec3::INFINITY,
            frame_in_flight: world.ctx.frame_in_flight(),
            last_image: 0,
            car_set: None,
            car_body: None,
            car_body_task: Some(car),
            car_body_colliders: vec![],
            body_handle: None,
            buffer: None,
            //left_wheel,
            //right_wheel,
            position: car_position,
            rotation: Quat::IDENTITY,
            // body_handle,
            // wheel_handles,
            // motor_joints,
            // steering_joints,
            max_steering_angle,
        })
    }

    pub fn load_if_finish(&mut self, world: &WorldRenderer, physics: &mut PhysicsEngine) {
        
        if self.car_body_task.as_ref().map_or(false, |h| h.is_finished()) {
            
            let model = self.car_body_task.take().unwrap().join().unwrap().unwrap();

            let mut car_body_colliders = vec![];

            let mut min = Vec3::splat(f32::MAX);
            let mut max = Vec3::splat(f32::MIN);

            for (vertices, _) in &model.physic_vertices {
                for v in vertices {
                    min = min.min(Vec3::new(v.x, v.y, v.z));
                    max = max.max(Vec3::new(v.x, v.y, v.z));
                }
            }

            let half_extents = (max - min) * 0.5;
            let center = (max + min) * 0.5;

            self.center = center;
            println!("cuboid half_extents: {:?}", half_extents);
            println!("center offset: {:?}", center);

            // Один rigid body
            let rigid_body = RigidBodyBuilder::dynamic()
                .translation(Vec3::new(0.0, 1.0, 0.0))
                .build();

            let rigid_handle = physics.create_rigid_body(rigid_body);

            // Один коллайдер с офсетом центра
            let collider = ColliderBuilder::cuboid(half_extents.x, half_extents.y, half_extents.z)
                .translation(Vec3::new(center.x, center.y, center.z))
                .restitution(0.5)
                .build();

            car_body_colliders.push(physics.create_collider(collider, rigid_handle));

            self.body_handle = Some(rigid_handle);

            let mut buffers = vec![];
            let mut sets = vec![];

            for i in 0..world.ctx.frame_in_flight() {

                let mut car_storage_buffer = GpuBufferBuilder::cpu_only(&world.ctx.device)
                    .size((size_of::<Transform>() * model.transforms.len()) as u64)
                    .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
                    .build()
                    .unwrap();

                car_storage_buffer.upload_data(&model.transforms).unwrap();

                let set_layout = DescriptorSetLayoutBuilder::new(&world.ctx.device)
                    .bindings(vec![
                        vk::DescriptorSetLayoutBinding::default()
                            .stage_flags(vk::ShaderStageFlags::VERTEX)
                            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                            .descriptor_count(1)
                    ])
                    .build()
                    .unwrap();

                let car_set = world.pool.create_descriptor_set(&world.ctx.device, &[set_layout.raw]).unwrap()[0];

                unsafe {

                    let buffer_info = vk::DescriptorBufferInfo::default()   
                        .buffer(car_storage_buffer.raw)
                        .offset(0)
                        .range(vk::WHOLE_SIZE);

                    let write = vk::WriteDescriptorSet::default()
                        .descriptor_count(1)
                        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                        .dst_set(car_set)
                        .dst_binding(0)
                        .dst_array_element(0)
                        .buffer_info(std::slice::from_ref(&buffer_info));

                    world.ctx.device.update_descriptor_sets(&[write], &[]);
                }

                
                buffers.push(car_storage_buffer);
                sets.push(car_set);
            }

            self.car_body = Some(model);
            self.buffer = Some(buffers);
            self.car_set = Some(sets);
            self.car_body_colliders = car_body_colliders;

        }
    }

    pub fn update_motors(
        &mut self,
        joints: &mut ImpulseJointSet,
        thrust: f32,
        steering: f32,
    ) {
        // let target_vel = -30.0 * thrust;

        // // Дифференциал
        // let diff = 2.0;
        // let shift = (self.max_steering_angle * steering).sin() * diff;
        // let speed_diff = if shift > 0.0 {
        //     f32::hypot(1.0, shift)
        // } else {
        //     1.0 / f32::hypot(1.0, shift)
        // };
        // let speeds = [target_vel / speed_diff, target_vel * speed_diff];

        // for (jh, &spd) in self.motor_joints.iter().zip(speeds.iter()) {
        //     if let Some(joint) = joints.get_mut(*jh, true) {
        //         joint.data.set_motor_velocity(JointAxis::AngX, spd, 100.0);
        //     }
        // }

        // // Педалирование
        // let target_angle = self.max_steering_angle * steering;
        // for jh in &self.steering_joints {
        //     if let Some(joint) = joints.get_mut(*jh, true) {
        //         joint.data.set_motor_position(
        //             JointAxis::AngY,
        //             target_angle,
        //             1e5,
        //             1e4,
        //         );
        //     }
        // }
    }

    pub fn sync_transforms(&mut self, physics: &PhysicsEngine) {

        for i in &self.car_body_colliders {

            let body = physics.rigid_body_set.get(self.body_handle.unwrap()).unwrap();
            let pos = body.translation();
            let rot = body.rotation();

            self.position = Vec3::new(pos.x, pos.y, pos.z);
            self.rotation = Quat::from_xyzw(rot.x, rot.y, rot.z, rot.w);

            let center_offset = Vec3::new(-self.center.x, -self.center.y / 2.0, -self.center.z);
            let physics_mat = Mat4::from_rotation_translation(self.rotation, self.position + center_offset);

            let mut transforms = vec![];

            if let Some(_) = self.car_body {
                for i in &self.car_body.as_ref().unwrap().transforms {

                    let original_mat = Mat4::from_cols_array_2d(&i.model);
                    let final_mat = physics_mat * original_mat;

                    transforms.push(Transform {
                        model: final_mat.to_cols_array_2d(),
                        normal: final_mat.inverse().transpose().to_cols_array_2d()
                    });
                }

                self.buffer.as_mut().unwrap()[(self.last_image + 1) as usize % self.frame_in_flight].upload_data(&transforms).unwrap();
            }
        }
    }
}
