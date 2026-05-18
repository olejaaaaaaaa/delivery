use winit::{application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent, event_loop::{self, ActiveEventLoop}, window::{CursorGrabMode, Fullscreen, Window, WindowId}};

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
                    game.handle_winit_event(window, event_loop, id, event);
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
        if let Some(game) = &mut self.game {
            game.handle_winit_device_event(event_loop, device_id, event);
        }
    }

    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {

        let window_attributes = Window::default_attributes()
            .with_title("Delivery")
            .with_inner_size(PhysicalSize::new(1280, 720));

        let window = event_loop
            .create_window(window_attributes)
            .expect("Error create window");

        window.set_cursor_visible(false);

        // let monitor = window.current_monitor().unwrap();

        // let video_mode = monitor
        //     .video_modes()
        //     .filter(|m| m.refresh_rate_millihertz() >= 59_000)
        //     .max_by_key(|m| (m.size().width, m.refresh_rate_millihertz()))
        //     .unwrap();

        //window.set_fullscreen(Some(Fullscreen::Exclusive(video_mode)));

        let game_renderer = GameRenderer::new(&window);
        self.game = Some(game_renderer);
        self.window = Some(window);
    }
}

