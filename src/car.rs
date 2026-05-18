use std::{sync::Arc, time::Instant};
use ash::vk;
use glam::{Mat4, Quat, Vec3};
use rapier3d::{dynamics::{GenericJointBuilder, ImpulseJointHandle, ImpulseJointSet, JointAxesMask, JointAxis, RigidBodyBuilder, RigidBodyHandle, RigidBodySet}, geometry::{ColliderBuilder, InteractionGroups}};
use winit::{event::{KeyEvent, WindowEvent}, event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowId}};
use crate::{BackgroundUiRenderer, Bindless, Camera, EguiRenderer, GltfModel, Loader2Renderer, LoaderRenderer, Mesh, PbrRenderer, PhysicsEngine, RenderLoop, SkyRenderer, Texture, WorldRenderer, core::{DescriptorPool, DescriptorPoolBuilder, DescriptorSetLayout, DescriptorSetLayoutBuilder, GpuBuffer, GpuBufferBuilder, TextureVertex, Vertex, VulkanResult}, load_gltf, open_vfs, render_context::GraphicsDevice};

use super::render_context::RenderContext;


pub struct Car {
    pub car_body_model: GltfModel,
    //left_wheel: GltfModel,
    //right_wheel: GltfModel,
    pub position: Vec3,
    pub rotation: Quat,
    pub body_handle: RigidBodyHandle,
    wheel_handles: [RigidBodyHandle; 4],
    motor_joints: Vec<ImpulseJointHandle>,
    steering_joints: Vec<ImpulseJointHandle>,
    max_steering_angle: f32,
}

impl Car {
    pub fn new(world: &WorldRenderer, physics: &mut PhysicsEngine) -> VulkanResult<Self> {

        let car = load_gltf(world, open_vfs(r"assets\meshes\apocalyptic_off_road_buggy.glb")).join().unwrap()?;

        // for (vertices, indices) in &car.physic_vertices {

        //     let indices: Vec<[u32; 3]> = indices
        //         .chunks(3)
        //         .map(|c| [c[0], c[1], c[2]])
        //         .collect();

        //     let collider = ColliderBuilder::trimesh(vertices.clone(), indices).unwrap().build();
        //     let rigid_body= RigidBodyBuilder::dynamic().build();
        //     let rigid_handle = physics.create_rigid_body(rigid_body);
        //     physics.create_collider(collider, rigid_handle);
        // }

        //let left_wheel = load_gltf(ctx, bindless, pool, open_vfs(r"C:\Users\Oleja\Desktop\cornell-mcray\assets\car\wheel_left.glb"))?;
        //let right_wheel = load_gltf(ctx, bindless, pool, open_vfs(r"C:\Users\Oleja\Desktop\cornell-mcray\assets\car\wheel_right.glb"))?;

        let car_position = Vec3::new(0.0, 1.0, 0.0);

        let max_steering_angle = 35.0f32.to_radians();
        let suspension_height = 0.12;

        // --- Кузов ---
        let body_rb = RigidBodyBuilder::dynamic()
            .translation(car_position + Vec3::new(0.0, 0.47, 0.0))
            .build();

        let body_handle = physics.create_rigid_body(body_rb);

        let body_col = ColliderBuilder::cuboid(0.65, 0.3, 0.9)
            .friction(0.8)
            .build();
        
        physics.create_collider(body_col, body_handle);

        // --- Колёса ---
        // (mesh_is_left, local_position_in_car_space, is_front)
        let wheel_params = [
            (Vec3::new(0.6874, 0.2783, -0.7802), false), // rear left
            (Vec3::new(-0.6874, 0.2783, -0.7802), false), // rear right
            (Vec3::new( 0.64,   0.2783,  1.0254), true),  // front left
            (Vec3::new(-0.64,   0.2783,  1.0254), true),  // front right
        ];

        let mut wheel_handles = [RigidBodyHandle::invalid(); 4];
        let mut motor_joints = vec![];
        let mut steering_joints = vec![];

        for (i, (local_pos, is_front)) in wheel_params.into_iter().enumerate() {
            let world_pos = car_position + local_pos;

            let wheel_rb = RigidBodyBuilder::dynamic()
                .translation(world_pos)
                .build();

            let wheel_handle = physics.create_rigid_body(wheel_rb);

            let wheel_col = ColliderBuilder::ball(0.28)
                .friction(1.2)
                .collision_groups(
                    InteractionGroups::new(
                        0x02.into(), 
                        0x01.into(),
                    rapier3d::geometry::InteractionTestMode::And
                    )
                )
                .build();

            physics.create_collider(wheel_col, wheel_handle);

            wheel_handles[i] = wheel_handle;

            let attachment = local_pos - Vec3::new(0.0, 0.47, 0.0);

            let mut joint = if is_front {
                GenericJointBuilder::new(JointAxesMask::ANG_X | JointAxesMask::ANG_Z)
                    .local_anchor1(attachment)
                    .local_anchor2(Vec3::ZERO)
                    .set_motor(JointAxis::LinY, 0.0, 0.0, 1e5, 1e4)
                    .limits(JointAxis::LinY, [-suspension_height, suspension_height])
                    .build()
            } else {
                GenericJointBuilder::new(
                    JointAxesMask::ANG_X | JointAxesMask::ANG_Y | JointAxesMask::ANG_Z
                )
                    .local_anchor1(attachment)
                    .local_anchor2(Vec3::ZERO)
                    .set_motor(JointAxis::LinY, 0.0, 0.0, 1e5, 1e4)
                    .limits(JointAxis::LinY, [-suspension_height, suspension_height])
                    .build()
            };

            // Мотор вращения колеса (задние)
            if !is_front {
                joint.set_motor(JointAxis::AngX, 0.0, 0.0, 0.0, 100.0);
                let jh = physics.create_joint(body_handle, wheel_handle, joint, true);
                motor_joints.push(jh);
            } else {
                joint.set_motor(JointAxis::AngY, 0.0, 0.0, 1e5, 1e4);
                let jh = physics.create_joint(body_handle, wheel_handle, joint, true);
                steering_joints.push(jh);
            }
        }

        Ok(Self {
            car_body_model: car,
            //left_wheel,
            //right_wheel,
            position: car_position,
            rotation: Quat::IDENTITY,
            body_handle,
            wheel_handles,
            motor_joints,
            steering_joints,
            max_steering_angle,
        })
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

        // Педалирование
        let target_angle = self.max_steering_angle * steering;
        for jh in &self.steering_joints {
            if let Some(joint) = joints.get_mut(*jh, true) {
                joint.data.set_motor_position(
                    JointAxis::AngY,
                    target_angle,
                    1e5,
                    1e4,
                );
            }
        }
    }

    pub fn sync_transforms(&mut self, bodies: &RigidBodySet) {
        if let Some(body) = bodies.get(self.body_handle) {
            let pos = body.translation();
            let rot = body.rotation();
            self.position = Vec3::new(pos.x, pos.y, pos.z);
            self.rotation = Quat::from_xyzw(rot.x, rot.y, rot.z, rot.w);

            // self.car_model.set_transform(0, 
            //     crate::Transform {
            //         model ()
            //         view: (), 
            //         proj: (), 
            //         normal: () 
            //     }
            // );
            // for t in &mut self.car_model.transforms_cpu {
            //     t.model = Mat4::from_rotation_translation(self.rotation, self.position).to_cols_array_2d();
            //     t.normal = Mat4::from_rotation_translation(self.rotation, self.position)
            //         .inverse().transpose().to_cols_array_2d();
            // }

            // self.car_model.ssbo.as_mut().unwrap()
            //     .upload_data(&self.car_model.transforms_cpu).unwrap();

        }
    }
}
