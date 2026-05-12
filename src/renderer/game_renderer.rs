use std::sync::Arc;
use ash::vk;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::{Window, WindowId}};
use crate::{Bindless, EguiRenderer, Mesh, PhysicsEngine, RenderLoop, SimpleRenderer, SimpleUiRenderer, Texture, core::{GpuBuffer, GpuBufferBuilder, TextureVertex, Vertex, VulkanResult}};

use super::super::render_context::RenderContext;



pub struct GameRenderer {
    phys_engine: PhysicsEngine,
    simple_mesh: Mesh,
    ui_mesh: Mesh,
    texture: Texture,
    bindless: Bindless,
    render_loop: RenderLoop,
    egui_renderer: EguiRenderer,
    simple_renderer: SimpleRenderer,
    simple_ui_renderer: SimpleUiRenderer,
    ctx: Arc<RenderContext>
}

impl GameRenderer {

    pub fn new(window: &Window) -> VulkanResult<Self> {
        let ctx = RenderContext::new(window).unwrap();

        let vertices = vec![
            Vertex { pos: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
            Vertex { pos: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
            Vertex { pos: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] }
        ];

        let mut vertex_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
            .size((size_of::<Vertex>() * vertices.len()) as u64)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .build()?;

        vertex_buffer.upload_data(&vertices)?;

        let indices = vec![0, 1, 2u32];
        
        let mut index_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
            .size((size_of::<u32>() * indices.len()) as u64)
            .usage(vk::BufferUsageFlags::INDEX_BUFFER)
            .build()?;

        index_buffer.upload_data(&indices)?;

        let simple_mesh = Mesh { 
            vertex_buffer,
            index_buffer,
            index_count: 3
        };

        let bindless = Bindless::new(&ctx)?;

        let simple_texture = image::open(r"assets\textures\texture0.jpg").unwrap();
        let rgba_simple_texture = simple_texture.to_rgba8();
        let (width, height) =  (rgba_simple_texture.width(), rgba_simple_texture.height());
        let pixels = rgba_simple_texture.into_raw();

        let mut texture = Texture::new(
            &ctx, 
            vk::Format::R8G8B8A8_SRGB, 
            width, 
            height, 
            &pixels
        )?;

        let index = bindless.alloc_texture(&ctx.device, texture.image_view.raw);
        texture.bindless_index = Some(index);

        let vertices = vec![
            TextureVertex { pos: [0.0, 0.2, 0.0],  uv: [0.5, 0.0] },
            TextureVertex { pos: [-0.2, -0.2, 0.0], uv: [0.0, 1.0] },
            TextureVertex { pos: [0.2, -0.2, 0.0],  uv: [1.0, 1.0] }
        ];

        let mut vertex_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
            .size((size_of::<Vertex>() * vertices.len()) as u64)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .build()?;

        vertex_buffer.upload_data(&vertices)?;

        let indices = vec![0, 1, 2u32];
        
        let mut index_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
            .size((size_of::<u32>() * indices.len()) as u64)
            .usage(vk::BufferUsageFlags::INDEX_BUFFER)
            .build()?;

        index_buffer.upload_data(&indices)?;

        let ui_mesh = Mesh {
            vertex_buffer,
            index_buffer,
            index_count: 3
        };

        Ok(Self { 
            phys_engine: PhysicsEngine::new(),
            egui_renderer: EguiRenderer::new(&ctx, window),
            render_loop: RenderLoop::new(&ctx),
            simple_ui_renderer: SimpleUiRenderer::new(&ctx, &bindless)?,
            simple_renderer: SimpleRenderer::new(&ctx)?,
            texture,
            simple_mesh,
            ui_mesh,
            ctx,
            bindless,
        })
    }
    
    pub fn handle_event(&mut self, winit_window: &Window, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {

        self.egui_renderer.egui_winit.on_window_event(winit_window, &event);

        match event {
            WindowEvent::Resized(size) => {
                let (width, height) = (size.width, size.height);
                self.ctx.resize(width, height).expect("Error resize window");
            },
            WindowEvent::RedrawRequested => {
                self.render_loop.draw(&self.ctx, |device, window, cmd, image_index| {

                    unsafe {
                        let clear_values = vec![
                            vk::ClearValue {
                                color: vk::ClearColorValue {
                                    float32: [0.0, 0.0, 0.0, 1.0],
                                },
                            },
                            vk::ClearValue {
                                depth_stencil: vk::ClearDepthStencilValue {
                                    depth: 1.0,
                                    stencil: 0,
                                },
                            }
                        ];

                        let frame_buffer = window.frame_buffers[image_index as usize].raw;
                            let render_pass_begin_info = vk::RenderPassBeginInfo::default()
                                .render_pass(window.render_pass.raw)
                                .framebuffer(frame_buffer)
                                .render_area(vk::Rect2D {
                                    offset: vk::Offset2D { x: 0, y: 0 },
                                    extent: window.resolution
                                })
                                .clear_values(&clear_values);

                        device.cmd_begin_render_pass(
                            cmd,
                            &render_pass_begin_info,
                            vk::SubpassContents::INLINE,
                        );
                    }

                    self.egui_renderer.draw(
                        cmd, 
                        winit_window
                    );

                    self.simple_renderer.draw_mesh(
                        device, 
                        window,
                        image_index,
                        cmd, 
                        &self.simple_mesh.vertex_buffer, 
                        &self.simple_mesh.index_buffer, 
                        self.simple_mesh.index_count
                    );

                    self.simple_ui_renderer.draw_ui(
                        device, 
                        window, 
                        &self.bindless,
                        image_index, 
                        cmd, 
                        &self.ui_mesh.vertex_buffer, 
                        &self.ui_mesh.index_buffer, 
                        self.ui_mesh.index_count
                    );

            

                    unsafe {
                        device.cmd_end_render_pass(cmd);
                    }

                }).unwrap();
            },
            _ => {},
        }
    }
}

impl Drop for GameRenderer {
    fn drop(&mut self) {
        
    }
}