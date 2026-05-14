use std::sync::Arc;

use ash::vk;
use egui::{Context, TextureId, ViewportId};
use egui_ash_renderer::{Options, Renderer};
use egui_winit::State;
use winit::window::Window;

use crate::{core::{CommandPool, CommandPoolBuilder, Queue}, render_context::RenderContext};


pub struct EguiRenderer {
    pub frame_time: f32,
    pub stats: Vec<(String, f32)>,
    pub queue: Queue,
    pub pool: CommandPool,
    pub egui_ctx: Context,
    pub egui_winit: State,
    pub renderer: Renderer,
    textures_to_free: Option<Vec<TextureId>>,
}

impl EguiRenderer {
    pub fn new(ctx: &Arc<RenderContext>, window: &Window) -> Self {
        
        let egui_ctx = Context::default();

        egui_extras::install_image_loaders(&egui_ctx);
        
        let egui_winit = State::new(
            egui_ctx.clone(),
            ViewportId::ROOT,
            &window,
            None,
            None,
            None,
        );

        let renderer = Renderer::with_default_allocator(
            &ctx.device.instance.raw,
            ctx.device.phys_dev.raw,
            ctx.device.raw.clone(),
            ctx.window.read().render_pass.raw,
            Options {
                srgb_framebuffer: true,
                ..Default::default()
            },
        ).unwrap();

        let queue = ctx.device.queue_pool.graphics().unwrap();
        let pool = CommandPoolBuilder::reset(&ctx.device).build().unwrap();

        Self { 
            stats: vec![],
            frame_time: 0.0,
            pool,
            queue, 
            egui_ctx, 
            egui_winit, 
            renderer, 
            textures_to_free: None 
        }
    }

    pub fn draw(&mut self, cmd: vk::CommandBuffer, window: &Window) {

        //Free last frames textures after the previous frame is done rendering
        if let Some(textures) = self.textures_to_free.take() {
            self.renderer
                .free_textures(textures.as_slice())
                .expect("Failed to free textures");
        }

        // Generate UI
        let raw_input = self.egui_winit.take_egui_input(&window);

        let egui::FullOutput {
            platform_output,
            textures_delta,
            shapes,
            pixels_per_point,
            ..
        } = self.egui_ctx.run(raw_input, |ctx| {
            egui::Window::new("Статистика")
                .resizable(true)        // можно изменять размер
                .movable(true)          // можно перетаскивать (по умолчанию true)
                .default_size([200.0, 100.0]) // начальный размер
                .show(ctx,  |ui| {
                    ui.label(format!("frame time: {:?}ms", self.frame_time));
                });
        });

        self.egui_winit
            .handle_platform_output(&window, platform_output);

        if !textures_delta.free.is_empty() {
            self.textures_to_free = Some(textures_delta.free.clone());
        }

        if !textures_delta.set.is_empty() {
            self.renderer
                .set_textures(
                    self.queue.raw,
                    self.pool.raw,
                    textures_delta.set.as_slice(),
                )
                .expect("Failed to update texture");
        }

        let primitives = self.egui_ctx.tessellate(shapes, pixels_per_point);

        let resolution = window.inner_size();

        self.renderer.cmd_draw(
            cmd, 
            ash::vk::Extent2D { width: resolution.width, height: resolution.height }, 
            pixels_per_point, 
            &primitives
        ).unwrap();
    }
}

