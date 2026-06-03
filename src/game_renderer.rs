use std::time::Instant;

use dolly::drivers::{Arm, LookAt, Position, Rotation, YawPitch};
use glam::Vec3;
use mint::Point3;
use rapier3d::{dynamics::ImpulseJoint, pipeline::{QueryFilter, QueryPipeline}};
use rodio::cpal::FromSample;
use winit::{event::{ElementState, KeyEvent, WindowEvent}, event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}, window::{CursorGrabMode, Window, WindowId}};
use crate::{Car, PhysicsEngine, Track, WorldRenderer, open_vfs, play_music};

struct InputState {
    w_pressed: bool,
    s_pressed: bool,
    a_pressed: bool,
    d_pressed: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            w_pressed: false,
            s_pressed: false,
            a_pressed: false,
            d_pressed: false,
        }
    }
}

pub struct GameRenderer {
    input_state: InputState,
    last_time: Instant,
    renderer: WorldRenderer,
    car: Car,
    track: Track,
    physics: PhysicsEngine,
}

impl GameRenderer {
    pub fn new(window: &Window) -> Self {

        play_music(open_vfs(r"sound://backgroud.mp3"));
        
        let world = WorldRenderer::new(window).expect("Error create World Renderer");
        let mut physics = PhysicsEngine::new();

        let track = Track::new(&world).unwrap();
        let car = Car::new(&world, &mut physics).unwrap();

        Self { 
            input_state: InputState::default(),
            last_time: Instant::now(),
            car,
            track,
            renderer: world, 
            physics 
        }
    }

    pub fn handle_winit_event(&mut self, window: &Window, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match &event {
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                if event.physical_key == PhysicalKey::Code(KeyCode::Escape) {
                    event_loop.exit();
                }

                   // Определяем, нажата или отпущена клавиша
                let pressed = event.state == ElementState::Pressed;
                
                // Обновляем состояние
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => self.input_state.w_pressed = pressed,
                    PhysicalKey::Code(KeyCode::KeyS) => self.input_state.s_pressed = pressed,
                    PhysicalKey::Code(KeyCode::KeyA) => self.input_state.a_pressed = pressed,
                    PhysicalKey::Code(KeyCode::KeyD) => self.input_state.d_pressed = pressed,
                    PhysicalKey::Code(KeyCode::KeyQ) => {
                        //self.car.position = Vec3::new(0.0, 10.0, 0.0);
                    }
                    _ => {}
                }
            },
            WindowEvent::Resized(size) => {
                let (width, height) = (size.width, size.height);
                self.renderer.resize(width, height);
            },
            WindowEvent::RedrawRequested => {

                let dt = self.last_time.elapsed().as_secs_f32();
                self.last_time = Instant::now();

               if let Some(_) = &self.car.car_body {

                    self.renderer.camera.rig.driver_mut::<Position>().position = Point3 {
                        x: self.car.position.x,
                        y: self.car.position.y,
                        z: self.car.position.z,
                    };

                    let car_quat = glam::Quat::from_xyzw(
                        self.car.rotation.x,
                        self.car.rotation.y,
                        self.car.rotation.z,
                        self.car.rotation.w,
                    );
                    let (car_yaw, _, _) = car_quat.to_euler(glam::EulerRot::YXZ);
                    let car_yaw_deg = car_yaw.to_degrees();

                    // YawPitch = yaw машины + offset мыши
                    let yaw_pitch = self.renderer.camera.rig.driver_mut::<YawPitch>();
                    yaw_pitch.yaw_degrees   = car_yaw_deg + self.renderer.camera.camera_yaw_offset;
                    yaw_pitch.pitch_degrees = self.renderer.camera.camera_pitch;

                    // LookAt чуть выше центра машины
                    self.renderer.camera.rig.driver_mut::<LookAt>().target = Point3 {
                        x: self.car.position.x,
                        y: self.car.position.y + 1.0,
                        z: self.car.position.z,
                    };

                    self.renderer.camera.rig.update(dt);

                    let cam_pos = self.renderer.camera.rig.final_transform.position;
                    let target = glam::Vec3::new(
                        self.car.position.x,
                        self.car.position.y + 1.0,
                        self.car.position.z,
                    );
                    let cam = glam::Vec3::new(cam_pos.x, cam_pos.y, cam_pos.z);

                    let dir = (cam - target).normalize();
                    let dist = (cam - target).length();

                    let ray = rapier3d::geometry::Ray::new(
                        Vec3::new(target.x, target.y, target.z),
                        Vec3::new(dir.x, dir.y, dir.z),
                    );

                    let filter = QueryFilter::default()
                        .exclude_rigid_body(self.car.body_handle.unwrap());

                    let query_pipeline = self.physics.broad_phase.as_query_pipeline(
                        self.physics.narrow_phase.query_dispatcher(),
                        &self.physics.rigid_body_set,
                        &self.physics.collider_set,
                        filter
                    );

                    let max_dist = 6.5_f32;

                    let desired_z = if let Some((_, toi)) = query_pipeline.cast_ray(&ray, max_dist, false) {
                        -((toi - 0.3_f32))
                    } else {
                        -6.5
                    };

                    let arm = self.renderer.camera.rig.driver_mut::<Arm>();
                    let current_z = arm.offset.z;

                    arm.offset.z = if desired_z > current_z {
                       
                        desired_z
                    } else {
                      
                        current_z + (desired_z - current_z) * (dt * 5.0).min(1.0)
                    };
   

                }
                
                let mut thrust = 0.0;
                let mut steering = 0.0;
                
                // W/S - тяга
                if self.input_state.w_pressed {
                    thrust = -1.4;
                } else if self.input_state.s_pressed {
                    thrust = 1.4;
                }

                if self.input_state.a_pressed {
                    steering = 1.1;
                } else if self.input_state.d_pressed {
                    steering = -1.1;
                }

                let phys_time = Instant::now();

                if self.track.model.is_some() {
                    self.physics.update(dt);
                    self.car.update_motors(&mut self.physics.impulse, thrust, steering);
                    self.car.sync_transforms((self.renderer.ctx.window.read().current_frame + 1) % self.renderer.ctx.device.frame_in_flight, &self.physics);           
                }

                self.renderer.egui_renderer.gpu_name = self.renderer.ctx.device.logical_device.props2.properties.device_name_as_c_str().unwrap().to_str().unwrap().to_string();
                self.renderer.egui_renderer.width = window.inner_size().width;
                self.renderer.egui_renderer.height = window.inner_size().height;
                self.renderer.egui_renderer.phys_time = phys_time.elapsed().as_secs_f32() * 1000.0;
                self.renderer.egui_renderer.phys_max_time = self.renderer.egui_renderer.phys_max_time.max(phys_time.elapsed().as_secs_f32() * 1000.0);

                self.track.load_if_finish(&self.renderer, &mut self.physics);
                self.car.load_if_finish(&self.renderer, &mut self.physics);
            
                //let t = std::time::Instant::now();
                self.renderer.draw(window, dt, &mut self.car, &self.track);
                //println!("draw: {}ms", t.elapsed().as_millis());
            }
            _ => {}
        }
        self.renderer.handle_event(window, event_loop, window_id, event);
    }
    
    pub fn handle_winit_device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        self.renderer.handle_device_event(&event);
    }
}

