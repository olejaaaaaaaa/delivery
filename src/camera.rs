use std::sync::Arc;

use ash::vk;
use bytemuck::Zeroable;
use dolly::{drivers::{Position, Smooth, YawPitch}, rig::CameraRig};
use glam::Vec3;

use crate::{core::{DescriptorPool, DescriptorSetLayoutBuilder, GpuBuffer, GpuBufferBuilder, VulkanResult}, render_context::RenderContext};

use dolly::prelude::*;
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct Camera {
    pub set: vk::DescriptorSet,
    pub last_data: CameraData,
    rig: CameraRig,
    pub buffer: GpuBuffer,
    move_speed: f32,
    mouse_sensitivity: f32,
    // состояние клавиш
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraData {
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
    pub pos:  [f32; 3],
    pub _pad: f32,
}

impl Camera {
    pub fn new(ctx: &Arc<RenderContext>, pool: &DescriptorPool) -> VulkanResult<Self> {
        let rig = CameraRig::builder()
            .with(Position::new(mint::Point3::from_slice(&[0.0, 1.0, 0.0])))
            .with(YawPitch::new().yaw_degrees(0.0).pitch_degrees(-15.0))
            .with(Smooth::new_position_rotation(0.5, 0.5))
            .build();

        let buffer = GpuBufferBuilder::cpu_only(&ctx.device)
            .size(size_of::<CameraData>() as u64)
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
            .build()?;

        
        let set_layout = DescriptorSetLayoutBuilder::new(&ctx.device)
            .bindings(vec![
                vk::DescriptorSetLayoutBinding::default()
                    .stage_flags(vk::ShaderStageFlags::VERTEX)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(1)
            ])
            .build()?;

        let set = pool.create_descriptor_set(&ctx.device, &[set_layout.raw])?[0];

        unsafe {

            let buffer_info = vk::DescriptorBufferInfo::default()   
                .buffer(buffer.raw)
                .offset(0)
                .range(vk::WHOLE_SIZE);

            let write = vk::WriteDescriptorSet::default()
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .dst_set(set)
                .dst_binding(0)
                .dst_array_element(0)
                .buffer_info(std::slice::from_ref(&buffer_info));

            ctx.device.update_descriptor_sets(&[write], &[]);
        }

        Ok(Self {
            set,
            rig,
            buffer,
            last_data: CameraData::zeroed(),
            move_speed: 5.0,
            mouse_sensitivity: 0.1,
            forward: false, backward: false,
            left: false, right: false,
            up: false, down: false,
        })
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        if let WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(key), state, .. }, .. } = event {
            let pressed = *state == ElementState::Pressed;
            match key {
                KeyCode::KeyW => self.forward  = pressed,
                KeyCode::KeyS => self.backward = pressed,
                KeyCode::KeyA => self.left     = pressed,
                KeyCode::KeyD => self.right    = pressed,
                KeyCode::KeyE => self.up       = pressed,
                KeyCode::KeyQ => self.down     = pressed,
                _ => {}
            }
        }
    }

    pub fn handle_device_event(&mut self, event: &DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            self.rig.driver_mut::<YawPitch>()
                .rotate_yaw_pitch(
                    -(*dx as f32) * self.mouse_sensitivity,
                    -(*dy as f32) * self.mouse_sensitivity,
                );
        }
    }

    pub fn update(&mut self, dt: f32, aspect: f32) -> CameraData {
        let mut move_vec = glam::Vec3::ZERO;
        if self.forward  { move_vec += glam::Vec3::Z * -1.0; }
        if self.backward { move_vec += glam::Vec3::Z; }
        if self.left     { move_vec += glam::Vec3::X * -1.0; }
        if self.right    { move_vec += glam::Vec3::X; }
        if self.up       { move_vec += glam::Vec3::Y; }
        if self.down     { move_vec += glam::Vec3::Y * -1.0; }

        if move_vec.length_squared() > 0.0 {
            let rot = self.rig.final_transform.rotation;
            // mint::Quaternion -> glam::Quat
            let rot = glam::Quat::from_xyzw(rot.v.x, rot.v.y, rot.v.z, rot.s);
            let move_vec = rot * move_vec.normalize();
            let v = move_vec * self.move_speed * dt;
            self.rig.driver_mut::<Position>()
                .translate(mint::Vector3::from_slice(&[v.x, v.y, v.z]));
        }

        let transform = self.rig.update(dt);

        let pos = transform.position;
        let pos_glam = glam::vec3(pos.x, pos.y, pos.z);

        // forward() тоже mint::Vector3, конвертим
        let fwd: mint::Vector3<f32> = transform.forward();
        let fwd_glam = glam::vec3(fwd.x, fwd.y, fwd.z);

        let view = glam::Mat4::look_at_rh(
            pos_glam,
            pos_glam + fwd_glam,
            glam::Vec3::Y,
        );

        let mut proj = glam::Mat4::perspective_rh(
            45_f32.to_radians(),
            aspect,
            0.01,
            1000.0,
        );

        proj.y_axis.y *= -1.0;

        let data = CameraData {
            view: view.to_cols_array_2d(),
            proj: proj.to_cols_array_2d(),
            pos:  pos.into(),
            _pad: 0.0,
        };

        //println!("data: {}");

        //println!("pos: {:?}", pos_glam);
        //println!("view: {:?}", view);

        self.last_data = data;
        self.buffer.upload_data(&[data]).unwrap();
        data
    }
}