use std::time::Instant;

use dolly::drivers::{LookAt, Position, Rotation};
use glam::Vec3;
use mint::Point3;
use winit::{event::{KeyEvent, WindowEvent}, event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}, window::{CursorGrabMode, Window, WindowId}};
use crate::{Car, PhysicsEngine, Track, WorldRenderer, open_vfs, play_music};

pub struct GameRenderer {
    global_time: Instant,
    world: WorldRenderer,
    car: Car,
    track: Track,
    physics: PhysicsEngine,
}

impl GameRenderer {
    pub fn new(window: &Window) -> Self {


        play_music(open_vfs(r"assets\sounds\backgroud.mp3"));
        let world = WorldRenderer::new(window).expect("Error create World Renderer");
        let mut physics = PhysicsEngine::new();

        let track = Track::new(&world).unwrap();
        let car = Car::new(&world, &mut physics).unwrap();

        Self { 
            global_time: Instant::now(),
            car,
            track,
            world, 
            physics 
        }
    }

    pub fn handle_winit_event(&mut self, winit_window: &Window, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match &event {
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                if event.physical_key == PhysicalKey::Code(KeyCode::Escape) {
                    event_loop.exit();
                }
            },
            WindowEvent::Resized(size) => {
                let (width, height) = (size.width, size.height);
                self.world.resize(width, height);
            },
            WindowEvent::RedrawRequested => {

                let dt = self.global_time.elapsed().as_secs_f32();
                self.global_time = Instant::now();

               if let Some(_) = &self.car.car_body {
                    let camera = &mut self.world.camera;
                    camera.rig.driver_mut::<Position>().position = Point3 {
                        x: self.car.position.x,
                        y: self.car.position.y,
                        z: self.car.position.z,
                    };
                    camera.rig.driver_mut::<Rotation>().rotation = mint::Quaternion {
                        v: mint::Vector3 {
                            x: self.car.rotation.x,
                            y: self.car.rotation.y,
                            z: self.car.rotation.z,
                        },
                        s: self.car.rotation.w,
                    };
                    camera.rig.driver_mut::<LookAt>().target = Point3 {
                        x: (self.car.position.x + Vec3::Y.x),
                        y: (self.car.position.y + Vec3::Y.y),
                        z: (self.car.position.z + Vec3::Y.z),
                    };
                }

                if self.track.model.is_some() {

                    let t = std::time::Instant::now();
                    self.physics.update(dt);
                    //println!("physics: {}ms", t.elapsed().as_millis());

                    let t = std::time::Instant::now();
                    self.car.sync_transforms(&self.physics);
                    //println!("sync: {}ms", t.elapsed().as_millis());
                }

                self.track.load_if_finish(&self.world, &mut self.physics);
                self.car.load_if_finish(&self.world, &mut self.physics);

                
                
                let t = std::time::Instant::now();
                self.world.draw(winit_window, &mut self.car, &self.track);
                //println!("draw: {}ms", t.elapsed().as_millis());
            }
            _ => {}
        }
        self.world.handle_event(winit_window, event_loop, window_id, event);
    }
    
    pub fn handle_winit_device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        self.world.handle_device_event(&event);
    }
}

