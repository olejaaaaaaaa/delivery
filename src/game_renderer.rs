use winit::{event::{KeyEvent, WindowEvent}, event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}, window::{CursorGrabMode, Window, WindowId}};
use crate::{Car, PhysicsEngine, Track, WorldRenderer, open_vfs, play_music};

pub struct GameRenderer {
    world: WorldRenderer,
    //car: Car,
    track: Track,
    physics: PhysicsEngine,
}

impl GameRenderer {
    pub fn new(window: &Window) -> Self {
        play_music(open_vfs(r"assets\sounds\backgroud.mp3"));
        let world = WorldRenderer::new(window).expect("Error create World Renderer");
        let mut physics = PhysicsEngine::new();

        let track = Track::new(&world).unwrap();
        //let car = Car::new(&world, &mut physics).unwrap();

        Self { 
            //car,
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
                self.physics.update();
                self.track.load_if_finish(&self.world, &mut self.physics);
                self.world.draw(winit_window, &self.track);
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

