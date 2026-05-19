use std::{sync::Arc, time::Instant};
use ash::vk;
use glam::{Mat4, Quat, Vec3};
use rapier3d::{dynamics::{GenericJointBuilder, ImpulseJointHandle, ImpulseJointSet, JointAxesMask, JointAxis, RigidBodyBuilder, RigidBodyHandle, RigidBodySet}, geometry::{ColliderBuilder, InteractionGroups}};
use winit::{event::{DeviceEvent, KeyEvent, WindowEvent}, event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowId}};
use crate::{BackgroundUiRenderer, Bindless, Camera, Car, EguiRenderer, GltfModel, Loader2Renderer, LoaderRenderer, Mesh, PbrRenderer, PhysicsEngine, RenderLoop, SkyRenderer, Texture, Track, core::{DescriptorPool, DescriptorPoolBuilder, DescriptorSetLayout, DescriptorSetLayoutBuilder, GpuBuffer, GpuBufferBuilder, TextureVertex, Vertex, VulkanResult}, load_gltf, open_vfs, render_context::GraphicsDevice, renderer::render_loop::FrameContext};

use super::super::render_context::RenderContext;

pub struct WorldRenderer {
    pub(crate) global_time: Instant,
    pub(crate) is_run: bool,
    pub(crate) debug_ui_enable: bool,
    pub(crate) camera: Camera,
    pub(crate) last_time: std::time::Instant,
    pub(crate) pool: DescriptorPool,
    pub(crate) bindless: Arc<Bindless>,
    pub(crate) render_loop: RenderLoop,
    pub(crate) egui_renderer: EguiRenderer,
    pub(crate) loader2_renderer: Loader2Renderer,
    pub(crate) loader_renderer: LoaderRenderer,
    pub(crate) pbr_renderer: PbrRenderer,
    pub(crate) sky_renderer: SkyRenderer,
    pub(crate) back_renderer: BackgroundUiRenderer,
    pub(crate) ctx: Arc<RenderContext>,
}

impl WorldRenderer {

    pub fn new(window: &Window) -> VulkanResult<Self> {
        let ctx = RenderContext::new(window).unwrap();
        let bindless = Bindless::new(&ctx)?;

        let pool = DescriptorPoolBuilder::new(&ctx.device)
            .max_sets(50)
            .pool_sizes(&[
                vk::DescriptorPoolSize::default()
                    .descriptor_count(5)
                    .ty(vk::DescriptorType::SAMPLED_IMAGE),
                vk::DescriptorPoolSize::default()
                    .descriptor_count(25)
                    .ty(vk::DescriptorType::STORAGE_BUFFER),
                vk::DescriptorPoolSize::default()
                    .descriptor_count(5)
                    .ty(vk::DescriptorType::STORAGE_IMAGE),
                vk::DescriptorPoolSize::default()
                    .descriptor_count(15)
                    .ty(vk::DescriptorType::UNIFORM_BUFFER),
            ])
            .build()?;

        let sky_renderer = SkyRenderer::new(&ctx, &pool)?;
        let back_renderer = BackgroundUiRenderer::new(&ctx, &bindless, &sky_renderer)?;

        Ok(Self { 
            is_run: false,
            global_time: Instant::now(),
            debug_ui_enable: true,
            last_time: Instant::now(),
            loader2_renderer: Loader2Renderer::new(&ctx)?,
            loader_renderer: LoaderRenderer::new(&ctx, &bindless)?,
            egui_renderer: EguiRenderer::new(&ctx, window),
            render_loop: RenderLoop::new(&ctx),
            camera: Camera::new(&ctx, &pool)?,
            pbr_renderer: PbrRenderer::new(&ctx, &bindless)?,
            back_renderer,
            sky_renderer,
            pool,
            ctx,
            bindless: Arc::new(bindless),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.ctx.resize(width, height).expect("Error resize window");
    }

    pub fn draw(&mut self, winit_window: &Window, car: &mut Car, track: &Track) {
        let dt = self.last_time.elapsed().as_secs_f32();
        self.egui_renderer.frame_time = self.last_time.elapsed().as_millis() as f32;
        self.last_time = std::time::Instant::now();

        let resolution = self.ctx.resolution();
        self.camera.update(dt, resolution.width as f32 / resolution.height as f32);

        self.render_loop.draw(&self.ctx, |frame| {

            if !self.sky_renderer.sky_rendered_once {
                transition_image_layout(&frame,
                    self.sky_renderer.cube_map.image.raw,
                    vk::ImageLayout::UNDEFINED,
                    vk::ImageLayout::GENERAL,
                );

                self.sky_renderer.dispatch_sky(&frame);

                transition_image_layout(&frame,
                    self.sky_renderer.cube_map.image.raw,
                    vk::ImageLayout::GENERAL,
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                );
                self.sky_renderer.sky_rendered_once = true;
            }

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

                let frame_buffer = frame.window.frame_buffers[frame.image_index as usize].raw;

                let render_pass_begin_info = vk::RenderPassBeginInfo::default()
                    .render_pass(frame.window.render_pass.raw)
                    .framebuffer(frame_buffer)
                    .render_area(vk::Rect2D {
                        offset: vk::Offset2D { x: 0, y: 0 },
                        extent: frame.window.resolution
                    })
                    .clear_values(&clear_values);

                frame.device.cmd_begin_render_pass(
                    frame.cmd,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                );
            }

            if track.model.is_none() || !self.is_run {
                //println!("One");
                self.loader2_renderer.draw_fullscreen(&frame, winit_window);
                self.loader_renderer.draw_fullscreen(
                    &frame, 
                    &self.bindless, 
                    winit_window
                );
            } else {

                //println!("Two");
                self.back_renderer.draw_fullscreen(
                    &frame,
                    &self.camera,
                    &self.bindless,
                    &self.sky_renderer
                );

                self.pbr_renderer.draw_mesh(
                    &frame,
                    &self.bindless, 
                    &self.camera,
                    car,
                    track
                );
            }

            car.last_image = frame.image_index;
            
            if self.debug_ui_enable {
                self.egui_renderer.draw(winit_window, &frame);
            }

            unsafe {
                frame.device.cmd_end_render_pass(frame.cmd);
            }

        }).unwrap();
    }

    pub fn handle_device_event(&mut self, event: &DeviceEvent) {
        self.camera.handle_device_event(event);
    }
    
    pub fn handle_event(&mut self, winit_window: &Window, event_loop: &ActiveEventLoop, _: WindowId, window_event: WindowEvent) {

        let _ = self.egui_renderer.egui_winit.on_window_event(winit_window, &window_event);

        self.camera.handle_event(&window_event);
        self.loader2_renderer.handle_event(winit_window, &window_event);

        match window_event {
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                if event.physical_key == PhysicalKey::Code(KeyCode::Tab) && event.state.is_pressed() {
                    self.debug_ui_enable = !self.debug_ui_enable;
                } else if event.physical_key == PhysicalKey::Code(KeyCode::KeyE) && event.state.is_pressed() {
                    self.is_run = true;
                }
            }
            _ => {},
        }
    }
}

fn transition_image_layout(
    frame: &FrameContext<'_>,
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
) {
    let (src_stage, src_access, dst_stage, dst_access) = match (old_layout, new_layout) {
        (vk::ImageLayout::UNDEFINED, vk::ImageLayout::GENERAL) => (
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::AccessFlags::empty(),
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::AccessFlags::SHADER_WRITE,
        ),
        (vk::ImageLayout::GENERAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::AccessFlags::SHADER_WRITE,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::AccessFlags::SHADER_READ,
        ),
        (vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, vk::ImageLayout::GENERAL) => (
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::AccessFlags::SHADER_READ,
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::AccessFlags::SHADER_WRITE,
        ),
        _ => unimplemented!()
    };

    let barrier = vk::ImageMemoryBarrier::default()
        .image(image)
        .old_layout(old_layout)   
        .new_layout(new_layout)       
        .src_access_mask(src_access) 
        .dst_access_mask(dst_access)  
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_array_layer(0)
                .layer_count(6)
                .base_mip_level(0)
                .level_count(1),
        );

    unsafe {
        frame.device.cmd_pipeline_barrier(
            frame.cmd,
            src_stage, 
            dst_stage,   
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        )
    };
}

impl Drop for WorldRenderer {
    fn drop(&mut self) {
        
    }
}