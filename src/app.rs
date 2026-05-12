use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::{self, ActiveEventLoop}, window::{Window, WindowId}};

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
                game.handle_event(event_loop, id, event);
            },
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Game")
            .with_inner_size(winit::dpi::LogicalSize::new(640.0, 480.0));

        let window = event_loop
            .create_window(window_attributes)
            .expect("Error create window");

        let game_renderer = GameRenderer::new(&window).unwrap();
        self.game = Some(game_renderer);
        self.window = Some(window);
    }
}

