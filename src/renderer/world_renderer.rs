use std::{collections::HashMap, hash::Hasher, sync::Arc, time::{Duration, Instant}};
use ash::vk;
use glam::{Mat4, Quat, Vec3};
use rapier3d::{dynamics::{GenericJointBuilder, ImpulseJointHandle, ImpulseJointSet, JointAxesMask, JointAxis, RigidBodyBuilder, RigidBodyHandle, RigidBodySet}, geometry::{ColliderBuilder, InteractionGroups}};
use winit::{event::{DeviceEvent, KeyEvent, WindowEvent}, event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowId}};
use crate::{BackgroundUiRenderer, Bindless, Camera, Car, EguiRenderer, GltfModel, Loader2Renderer, LoaderRenderer, Mesh, PbrRenderer, PhysicsEngine, RenderLoop, Resources, SkyRenderer, Texture, Track, core::{DescriptorPool, DescriptorPoolBuilder, DescriptorSetLayout, DescriptorSetLayoutBuilder, Device, GpuBuffer, GpuBufferBuilder, TextureVertex, Vertex, VulkanResult}, load_gltf, open_vfs, render_context::GraphicsDevice, renderer::render_loop::FrameContext};

use frame_graph::*;

use super::super::render_context::RenderContext;

pub struct WorldRenderer {
    pub(crate) graph: FrameGraph,
    pub(crate) is_run: bool,
    pub(crate) global_time: Instant,
    pub(crate) debug_ui_enable: bool,
    pub(crate) camera: Camera,
    pub(crate) render_loop: RenderLoop,
    pub(crate) egui_renderer: EguiRenderer,
    pub(crate) loader2_renderer: Loader2Renderer,
    pub(crate) loader_renderer: LoaderRenderer,
    pub(crate) pbr_renderer: PbrRenderer,
    pub(crate) sky_renderer: SkyRenderer,
    pub(crate) back_renderer: BackgroundUiRenderer,
    pub(crate) ctx: Arc<RenderContext>,
    pub(crate) resources: Arc<Resources>
}

impl WorldRenderer {

    pub fn new(window: &Window) -> VulkanResult<Self> {

        let ctx = RenderContext::new(window)?;
        let res = Resources::new(&ctx)?;

        let sky_renderer = SkyRenderer::new(&ctx, &res)?;
        let back_renderer = BackgroundUiRenderer::new(&ctx, &res, &sky_renderer)?;

        let mut graph = FrameGraph::new(&FrameGraphCreateDesc {
            allocator: ctx.device.allocator.clone(), 
            frame_in_flight: ctx.frame_in_flight(), 
            device: ctx.device.raw.clone()
        });

        Ok(Self { 
            graph,
            is_run: false,
            global_time: Instant::now().checked_add(Duration::from_secs_f32(1.0)).unwrap(),
            debug_ui_enable: false,
            loader2_renderer: Loader2Renderer::new(&ctx)?,
            loader_renderer: LoaderRenderer::new(&ctx, &res)?,
            egui_renderer: EguiRenderer::new(&ctx, window),
            render_loop: RenderLoop::new(&ctx),
            camera: Camera::new(&ctx, &res)?,
            pbr_renderer: PbrRenderer::new(&ctx, &res)?,
            back_renderer,
            sky_renderer,
            resources: res,
            ctx,
        })
    }

    fn update_buffers(&mut self, image_index: u32) {

    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.ctx.resize(width, height).expect("Error resize window");
    }

    pub fn draw(&mut self, window: &Window, dt: f32, car: &mut Car, track: &Track) {
  
        self.egui_renderer.frame_time = dt * 1000.0;

        let resolution = self.ctx.resolution();

        self.camera.update(dt, resolution.width as f32 / resolution.height as f32, (self.ctx.window.read().current_frame + 1) % self.ctx.device.frame_in_flight);

        self.render_loop.draw(&self.ctx, |frame| {

            let mut scope = FrameScope::new(&mut self.graph);

            
            //let skybox = scope.import( self.sky_renderer.cube_map.image.raw, TextureAccess::Undefined);
            
            if !self.sky_renderer.sky_rendered_once {

                self.sky_renderer.sky_rendered_once = true;
                transition_image_layout(&frame,
                    self.sky_renderer.cube_map.image.raw,
                    vk::ImageLayout::UNDEFINED,
                    vk::ImageLayout::GENERAL,
                );

                // scope.add_pass(
                //     Pass::new("SkyBox")
                //         .read(skybox.clone(), TextureAccess::ComputeWrite)
                //         .execute(|device, cmd, res| {
                //            
                //         })
                // );
    
                 self.sky_renderer.dispatch_sky(frame.device, frame.cmd);

                transition_image_layout(&frame,
                    self.sky_renderer.cube_map.image.raw,
                    vk::ImageLayout::GENERAL,
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                );
                
            }

            let backbuffer = scope.import(frame.window.swapchain_images[frame.image_index as usize], TextureAccess::Undefined);
            let depth = scope.import(frame.window.depth_images[frame.image_index as usize].raw, TextureAccess::Undefined);

            scope.add_pass(
                Pass::new("Main Pass")
                    .write(backbuffer.clone(), TextureAccess::ColorWrite)
                    .write(depth, TextureAccess::DepthWrite)
                    //.read(skybox, TextureAccess::FragmentRead)
                    .execute(|device, cmd, res| {

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
                        
                        unsafe {

                            let swapchain_view = &frame.window.image_views[frame.image_index as usize];

                            let color_attach = vk::RenderingAttachmentInfoKHR::default()
                                .load_op(vk::AttachmentLoadOp::CLEAR)
                                .store_op(vk::AttachmentStoreOp::STORE)
                                .image_view(swapchain_view.raw)
                                .clear_value(clear_values[0])
                                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

                            let depth_attach = vk::RenderingAttachmentInfoKHR::default()
                                .load_op(vk::AttachmentLoadOp::CLEAR)
                                .store_op(vk::AttachmentStoreOp::STORE)
                                .image_view(frame.window.depth_views[frame.image_index as usize].raw)
                                .clear_value(clear_values[1])
                                .image_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

                            let render_info = vk::RenderingInfoKHR::default()
                                .render_area(vk::Rect2D {
                                    offset: vk::Offset2D { x: 0, y: 0 },
                                    extent: frame.window.resolution
                                })
                                .depth_attachment(&depth_attach)
                                .color_attachments(std::slice::from_ref(&color_attach))
                                .layer_count(1);

                            frame.device.cmd_begin_rendering(cmd, &render_info);

                            if track.model.is_none() || !self.is_run {
                                self.loader2_renderer.draw_fullscreen(&frame, window);
                                self.loader_renderer.draw_fullscreen(
                                    &frame, 
                                    &self.resources.bindless, 
                                window
                                );
                            } else {
                                self.back_renderer.draw_fullscreen(
                                    &frame,
                                    &self.camera,
                                    &self.resources.bindless,
                                    &self.sky_renderer
                                );

                                self.pbr_renderer.draw_mesh(
                                    &frame,
                                    &self.resources.bindless, 
                                    &self.sky_renderer,
                                    &self.camera,
                                    car,
                                    track
                                );
                            }

                             if self.debug_ui_enable {
                                self.egui_renderer.draw(window, &frame);
                            }
                            
                            frame.device.cmd_end_rendering(frame.cmd);

                            car.last_image = frame.image_index;
                            
                        }

                    })

                
            );

            let _ = scope.export(backbuffer, TextureAccess::Present);
            scope.compile();
            scope.execute(frame.cmd);

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

fn transition_image_layout2(
    frame: &FrameContext<'_>,
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
) {
    let barrier = vk::ImageMemoryBarrier::default()
        .image(image)
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_access_mask(get_access_mask(old_layout))
        .dst_access_mask(get_access_mask(new_layout))
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(match new_layout {
                    vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL => 
                        vk::ImageAspectFlags::DEPTH,
                    _ => vk::ImageAspectFlags::COLOR,
                })
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1),
        );

    let src_stage = get_pipeline_stage(old_layout);
    let dst_stage = get_pipeline_stage(new_layout);

    unsafe {
        frame.device.cmd_pipeline_barrier(
            frame.cmd,
            src_stage,
            dst_stage,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );
    }
}

fn get_access_mask(layout: vk::ImageLayout) -> vk::AccessFlags {
    match layout {
        vk::ImageLayout::UNDEFINED => vk::AccessFlags::NONE,
        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL => 
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL => 
            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL => 
            vk::AccessFlags::SHADER_READ,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL => 
            vk::AccessFlags::TRANSFER_WRITE,
        vk::ImageLayout::PRESENT_SRC_KHR => vk::AccessFlags::NONE,
        _ => vk::AccessFlags::NONE,
    }
}

fn get_pipeline_stage(layout: vk::ImageLayout) -> vk::PipelineStageFlags {
    match layout {
        vk::ImageLayout::UNDEFINED => vk::PipelineStageFlags::TOP_OF_PIPE,
        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL => 
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL => 
            vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL => 
            vk::PipelineStageFlags::FRAGMENT_SHADER,
        vk::ImageLayout::PRESENT_SRC_KHR => 
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
        _ => vk::PipelineStageFlags::ALL_COMMANDS,
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