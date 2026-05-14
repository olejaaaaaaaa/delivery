use winit::{application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent, event_loop::{self, ActiveEventLoop}, window::{Fullscreen, Window, WindowId}};

use crate::GameRenderer;

#[derive(Default)]
pub struct App {
    game: Option<GameRenderer>,
    window: Option<winit::window::Window>,
}

impl ApplicationHandler for App {
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            _ => {
                let game = self.game.as_mut().unwrap();
                if let Some(window) = self.window.as_ref() {
                    game.handle_event(window, event_loop, id, event);
                }
            },
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    )
    {
        if let Some(window) = self.window.as_ref() {
            self.game.as_mut().unwrap().camera.handle_device_event(&event);
        }
    }

    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {

        let window_attributes = Window::default_attributes()
            .with_title("Delivery")
            .with_inner_size(PhysicalSize::new(1280, 720));

        let window = event_loop
            .create_window(window_attributes)
            .expect("Error create window");

        // let monitor = window.current_monitor().unwrap();
        // let video_mode = monitor.video_modes().next().unwrap();

        // window.set_fullscreen(Some(Fullscreen::Exclusive(video_mode)));

        let game_renderer = GameRenderer::new(&window).unwrap();
        self.game = Some(game_renderer);
        self.window = Some(window);
    }
}

