use std::{path::PathBuf, sync::Arc, thread::JoinHandle, time::Instant};
use ash::vk;
use glam::{Mat4, Quat, Vec3};
use rapier3d::{dynamics::{GenericJointBuilder, ImpulseJointHandle, ImpulseJointSet, JointAxesMask, JointAxis, RigidBodyBuilder, RigidBodyHandle, RigidBodySet}, geometry::{ColliderBuilder, ColliderHandle, Group, InteractionGroups, InteractionTestMode}};
use winit::{event::{KeyEvent, WindowEvent}, event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowId}};
use crate::{BackgroundUiRenderer, Bindless, Camera, EguiRenderer, GltfModel, Loader2Renderer, LoaderRenderer, Mesh, PbrRenderer, PhysicsEngine, RenderLoop, SkyRenderer, Texture, Transform, WorldRenderer, core::{DescriptorPool, DescriptorPoolBuilder, DescriptorSetLayout, DescriptorSetLayoutBuilder, GpuBuffer, GpuBufferBuilder, TextureVertex, Vertex, VulkanResult}, load_gltf, open_vfs, render_context::GraphicsDevice};

use super::render_context::RenderContext;


pub struct Wheel {
    pub axle_handle: RigidBodyHandle,    // без коллизии, для подвески
    pub wheel_handle: RigidBodyHandle,   // с коллизией и трением
    pub collider_handle: ColliderHandle,
    pub set: vk::DescriptorSet,
    pub model: GltfModel,
    pub buffer: GpuBuffer,
    pub center: Vec3,
    pub position: Vec3,
    pub rotation: Quat,
}
 
impl Wheel {
    pub fn new(
        world: &WorldRenderer,
        physics: &mut PhysicsEngine,
        path: PathBuf,
        spawn_pos: Vec3,
    ) -> VulkanResult<Self> {
        let model = load_gltf(world, path).join().unwrap()?;
 
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
 
        for (vertices, _) in &model.physic_vertices {
            for v in vertices {
                min = min.min(Vec3::new(v.x, v.y, v.z));
                max = max.max(Vec3::new(v.x, v.y, v.z));
            }
        }
 
        let center = (max + min) * 0.5;
 
        // Axle — sensor, без физической коллизии (только для подвески)
        let axle_rb = RigidBodyBuilder::dynamic().translation(spawn_pos).build();
        let axle_handle = physics.create_rigid_body(axle_rb);
        let axle_col = ColliderBuilder::ball(0.28).sensor(true).build();
        physics.create_collider(axle_col, axle_handle);
 
        // Wheel — с коллизией и трением, не бьётся с кузовом (группа 0x02 vs 0x01)
        let wheel_rb = RigidBodyBuilder::dynamic().translation(spawn_pos).enabled(false).build();
        let wheel_handle = physics.create_rigid_body(wheel_rb);

        let wheel_col = ColliderBuilder::ball(0.28)
            .friction(1.2)
            .collision_groups(InteractionGroups::new(Group::GROUP_2, Group::GROUP_1, InteractionTestMode::And))
            .build();

        let collider_handle = physics.create_collider(wheel_col, wheel_handle);
 
        // GPU буфер
        let mut storage_buffer = GpuBufferBuilder::cpu_only(&world.ctx.device)
            .size((size_of::<Transform>() * model.transforms.len()) as u64)
            .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
            .build()
            .unwrap();
        storage_buffer.upload_data(&model.transforms).unwrap();
 
        let set_layout = DescriptorSetLayoutBuilder::new(&world.ctx.device)
            .bindings(&[vk::DescriptorSetLayoutBinding::default()
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)])
            .build()
            .unwrap();
 
        let set = world.resources.pool
            .create_descriptor_set(&world.ctx.device, &[set_layout.raw])
            .unwrap()[0];
 
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
 
        Ok(Self {
            axle_handle,
            wheel_handle,
            collider_handle,
            set,
            model,
            position: spawn_pos,
            rotation: Quat::IDENTITY,
            center,
            buffer: storage_buffer,
        })
    }
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
    pub center: Vec3,
    pub position: Vec3,
    pub rotation: Quat,
    pub wheels: Vec<Wheel>,
    pub wheel_offsets: Vec<(Vec3, bool)>,  // (локальная поз., is_front)
    pub motor_joints: Vec<ImpulseJointHandle>,
    pub steering_joints: Vec<ImpulseJointHandle>,
    max_steering_angle: f32,
    suspension_height: f32,
}
 
impl Car {
    pub fn new(world: &WorldRenderer, physics: &mut PhysicsEngine) -> VulkanResult<Self> {
        let car = load_gltf(world, open_vfs(r"assets\meshes\336_lrm\car.glb"));
 
        let car_position = Vec3::new(0.0, 1.0, 0.0);
        let max_steering_angle = 35.0f32.to_radians();
        let suspension_height = 0.12;
 
        // (локальная позиция относительно машины, is_front)
        let wheel_offsets: Vec<(Vec3, bool)> = vec![
            (Vec3::new( 0.6874, 0.2783, -0.7802), false), // rear left
            (Vec3::new(-0.6874, 0.2783, -0.7802), false), // rear right
            (Vec3::new( 0.64,   0.2783,  1.0254), true),  // front left
            (Vec3::new(-0.64,   0.2783,  1.0254), true),  // front right
        ];
 
        let wheel_paths = [
            open_vfs(r"assets\meshes\336_lrm\wheel_left_up.glb"),
            open_vfs(r"assets\meshes\336_lrm\wheel_right_up.glb"),
            open_vfs(r"assets\meshes\336_lrm\wheel_left_down.glb"),
            open_vfs(r"assets\meshes\336_lrm\wheel_right_down.glb"),
        ];
 
        let mut wheels = vec![];
        for (i, (offset, _)) in wheel_offsets.iter().enumerate() {
            let spawn_pos = car_position + *offset;
            wheels.push(Wheel::new(world, physics, wheel_paths[i].clone(), spawn_pos)?);
        }
 
        Ok(Self {
            wheels,
            wheel_offsets,
            motor_joints: vec![],
            steering_joints: vec![],
            center: Vec3::INFINITY,
            frame_in_flight: world.ctx.frame_in_flight(),
            last_image: 0,
            car_set: None,
            car_body: None,
            car_body_task: Some(car),
            car_body_colliders: vec![],
            body_handle: None,
            buffer: None,
            position: car_position,
            rotation: Quat::IDENTITY,
            max_steering_angle,
            suspension_height,
        })
    }
 
    pub fn load_if_finish(&mut self, world: &WorldRenderer, physics: &mut PhysicsEngine) {
        if !self.car_body_task.as_ref().map_or(false, |h| h.is_finished()) {
            return;
        }

        let body_position_in_car_space = Vec3::new(0.0, 0.4739, 0.0);

        for (wheel, (local_pos, is_front)) in self.wheels.iter().zip(self.wheel_offsets.iter()) {
            let axle_body = physics.rigid_body_set.get(wheel.axle_handle).unwrap();
            let wheel_body = physics.rigid_body_set.get(wheel.wheel_handle).unwrap();
            println!("axle pos: {:?}", axle_body.translation());
            println!("wheel pos: {:?}", wheel_body.translation());
            println!("body pos: {:?}", self.position);
            println!("expected attachment: {:?}", local_pos);

            let attachment = *local_pos - body_position_in_car_space;
            println!("attachment local: {:?}", attachment);
        }
 
        let model = self.car_body_task.take().unwrap().join().unwrap().unwrap();
 
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
 
        let body_position_in_car_space = Vec3::new(0.0, 0.4739, 0.0);
        let body_position = self.position + body_position_in_car_space;
 
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(body_position) 
            .build();

        let rigid_handle = physics.create_rigid_body(rigid_body);

         let body = physics.rigid_body_set.get(rigid_handle).unwrap();
        println!("body actual pos: {:?}", body.translation());
 
        // Кузов — группа 0x01, колёса (0x02) с ним не сталкиваются
        let collider = ColliderBuilder::cuboid(half_extents.x, half_extents.y, half_extents.z)
            //.translation(Vec3::new(center.x, center.y, center.z))
            .collision_groups(InteractionGroups::new(Group::GROUP_1, Group::GROUP_1, InteractionTestMode::And))
            .friction(0.8)
            .restitution(0.5)
            .build();
 
        self.car_body_colliders.push(physics.create_collider(collider, rigid_handle));
        self.body_handle = Some(rigid_handle);
 
        // --- Джоинты для колёс ---
        // Создаём здесь, потому что только сейчас есть rigid_handle кузова
        let suspension_height = self.suspension_height;
        let max_steering_angle = self.max_steering_angle;
 
        for (wheel, (local_pos, is_front)) in
            self.wheels.iter().zip(self.wheel_offsets.iter())
        {

            let correct_pos = self.position + *local_pos;
            if let Some(rb) = physics.rigid_body_set.get_mut(wheel.axle_handle) {
                rb.set_translation(correct_pos, true);
            }
            if let Some(rb) = physics.rigid_body_set.get_mut(wheel.wheel_handle) {
                rb.set_translation(correct_pos, true);
            }

            println!("Setting wheel position to: {:?}", correct_pos);
            println!("Wheel local offset: {:?}", local_pos);
            println!("Body position: {:?}", self.position);
            println!("Expected attachment point: {:?}", *local_pos - body_position_in_car_space);

            let attachment = *local_pos - body_position_in_car_space;
 
            // Suspension: body → axle
            // Блокируем XZ трансляцию и все вращения кроме Y (руль у передних)
            let locked = JointAxesMask::LIN_X
                | JointAxesMask::LIN_Z
                | JointAxesMask::ANG_X
                | JointAxesMask::ANG_Z
                | if *is_front { JointAxesMask::empty() } else { JointAxesMask::ANG_Y };
 
            let suspension = GenericJointBuilder::new(locked)
                .local_anchor1(attachment)
                .local_anchor2(Vec3::ZERO)
                .limits(JointAxis::LinY, [-suspension_height, suspension_height])
                .set_motor(JointAxis::LinY, -suspension_height, 0.0, 1e5, 1e4)
                .build();
 
            let susp_handle = physics.create_joint(rigid_handle, wheel.axle_handle, suspension, true);
 
            if *is_front {
                self.steering_joints.push(susp_handle);
            }
 
            // Motor: axle → wheel
            // ANG_X свободен — это ось вращения колеса
            let motor_locked = JointAxesMask::LIN_X
                | JointAxesMask::LIN_Y
                | JointAxesMask::LIN_Z
                | JointAxesMask::ANG_Y
                | JointAxesMask::ANG_Z;
 
            let mut motor_joint = GenericJointBuilder::new(motor_locked)
                .local_anchor1(Vec3::ZERO)
                .local_anchor2(Vec3::ZERO)
                .build();
 
            if !*is_front {
                // Задние колёса — мотор по ANG_X
                motor_joint.set_motor(JointAxis::AngX, 0.0, 0.0, 0.0, 100.0);
                let motor_handle =
                    physics.create_joint(wheel.axle_handle, wheel.wheel_handle, motor_joint, true);
                self.motor_joints.push(motor_handle);
            } else {
                physics.create_joint(wheel.axle_handle, wheel.wheel_handle, motor_joint, true);
            }
        }
 
        // --- GPU буферы для кузова ---
        let mut buffers = vec![];
        let mut sets = vec![];
 
        for _ in 0..world.ctx.frame_in_flight() {
            let mut car_storage_buffer = GpuBufferBuilder::cpu_only(&world.ctx.device)
                .size((size_of::<Transform>() * model.transforms.len()) as u64)
                .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
                .build()
                .unwrap();
 
            car_storage_buffer.upload_data(&model.transforms).unwrap();
 
            let set_layout = DescriptorSetLayoutBuilder::new(&world.ctx.device)
                .bindings(&[vk::DescriptorSetLayoutBinding::default()
                    .stage_flags(vk::ShaderStageFlags::VERTEX)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .descriptor_count(1)])
                .build()
                .unwrap();
 
            let car_set = world.resources.pool
                .create_descriptor_set(&world.ctx.device, &[set_layout.raw])
                .unwrap()[0];
 
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

            for wheel in &self.wheels {
                if let Some(rb) = physics.rigid_body_set.get_mut(wheel.wheel_handle) {
                    rb.set_enabled(true);
                }
            }
        }
 
        self.car_body = Some(model);
        self.buffer = Some(buffers);
        self.car_set = Some(sets);
    }
 
    pub fn update_motors(
        &mut self,
        joints: &mut ImpulseJointSet,
        thrust: f32,
        steering: f32,
    ) {
        let target_vel = -30.0 * thrust;
 
        // Дифференциал
        let diff = 2.0;
        let shift = (self.max_steering_angle * steering).sin() * diff;
        let speed_diff = if shift > 0.0 {
            f32::hypot(1.0, shift)
        } else {
            1.0 / f32::hypot(1.0, shift)
        };
        let speeds = [target_vel / speed_diff, target_vel * speed_diff];
 
        for (jh, &spd) in self.motor_joints.iter().zip(speeds.iter()) {
            if let Some(joint) = joints.get_mut(*jh, true) {
                joint.data.set_motor_velocity(JointAxis::AngX, spd, 100.0);
            }
        }
 
        // Руль
        let target_angle = self.max_steering_angle * steering;
        for jh in &self.steering_joints {
            if let Some(joint) = joints.get_mut(*jh, true) {
                joint.data.set_motor_position(JointAxis::AngY, target_angle, 1e5, 1e4);
            }
        }
    }
 
    pub fn sync_transforms(&mut self, physics: &PhysicsEngine) {
        // --- Кузов ---
        if let Some(body_handle) = self.body_handle {
            let body = physics.rigid_body_set.get(body_handle).unwrap();
            let pos = body.translation();
            let rot = body.rotation();

            println!("body pos: {:?}", self.position);
 
            self.position = Vec3::new(pos.x, pos.y, pos.z);
            self.rotation = Quat::from_xyzw(rot.x, rot.y, rot.z, rot.w);
 
            if let Some(car_body) = &self.car_body {
                let center_offset =
                    Vec3::new(-self.center.x, -self.center.y / 2.0, -self.center.z);
                let physics_mat =
                    Mat4::from_rotation_translation(self.rotation, self.position + center_offset);
 
                let transforms: Vec<Transform> = car_body
                    .transforms
                    .iter()
                    .map(|t| {
                        let original_mat = Mat4::from_cols_array_2d(&t.model);
                        let final_mat = physics_mat * original_mat;
                        Transform {
                            model: final_mat.to_cols_array_2d(),
                            normal: final_mat.inverse().transpose().to_cols_array_2d(),
                        }
                    })
                    .collect();
 
                self.buffer.as_mut().unwrap()
                    [(self.last_image + 1) as usize % self.frame_in_flight]
                    .upload_data(&transforms)
                    .unwrap();
            }
        }
 
        // --- Колёса ---
        // Берём позицию из wheel_handle (у него коллизия, он реально едет по земле)
        for wheel in &mut self.wheels {
            let body = physics.rigid_body_set.get(wheel.wheel_handle).unwrap();
            let pos = body.translation();
            let rot = body.rotation();
 
            wheel.position = Vec3::new(pos.x, pos.y, pos.z);
            wheel.rotation = Quat::from_xyzw(rot.x, rot.y, rot.z, rot.w);

            println!("wheel pos: {:?}", wheel.center);
            println!("wheel rot: {:?}", wheel.rotation);
 
            let center_offset = Mat4::from_translation(wheel.center);
            let physics_mat = Mat4::from_rotation_translation(wheel.rotation, wheel.position);

            let transforms: Vec<Transform> = wheel.model.transforms.iter().map(|t| {
                let original_mat = Mat4::from_cols_array_2d(&t.model);
                let final_mat = original_mat;
                Transform {
                    model: final_mat.to_cols_array_2d(),
                    normal: final_mat.inverse().transpose().to_cols_array_2d(),
                }
            }).collect();

            wheel.buffer.upload_data(&transforms).unwrap();
        }
    }
}