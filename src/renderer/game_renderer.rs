use std::{sync::Arc, time::Instant};
use ash::vk;
use tracing_subscriber::fmt::layer;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::{Window, WindowId}};
use crate::{Bindless, Camera, EguiRenderer, GltfModel, Mesh, PbrRenderer, PhysicsEngine, RenderLoop, SimpleUiRenderer, SkyRenderer, Texture, core::{DescriptorPool, DescriptorPoolBuilder, DescriptorSetLayout, DescriptorSetLayoutBuilder, GpuBuffer, GpuBufferBuilder, TextureVertex, Vertex, VulkanResult}, load_gltf, open_vfs, render_context::GraphicsDevice};

use super::super::render_context::RenderContext;



pub struct GameRenderer {
    pub(crate) camera: Camera,
    last_time: std::time::Instant,
    phys_engine: PhysicsEngine,
    pool: DescriptorPool,
    set_layout_storage: DescriptorSetLayout,
    pbr_model: GltfModel,
    //simple_mesh: Mesh,
    //ui_mesh: Mesh,
    //texture: Texture,
    bindless: Bindless,
    render_loop: RenderLoop,
    egui_renderer: EguiRenderer,
    pbr_renderer: PbrRenderer,
    sky_renderer: SkyRenderer,
    //simple_renderer: SimpleRenderer,
    simple_ui_renderer: SimpleUiRenderer,
    ctx: Arc<RenderContext>,
    camera_set_layout: DescriptorSetLayout,
    camera_set: vk::DescriptorSet,
    set: vk::DescriptorSet
}

impl GameRenderer {

    pub fn new(window: &Window) -> VulkanResult<Self> {
        let ctx = RenderContext::new(window).unwrap();
        let bindless = Bindless::new(&ctx)?;

        // let simple_texture = image::open(r"assets\textures\texture0.jpg").unwrap();
        // let rgba_simple_texture = simple_texture.to_rgba8();
        // let (width, height) =  (rgba_simple_texture.width(), rgba_simple_texture.height());
        // let pixels = rgba_simple_texture.into_raw();

        // let mut texture = Texture::new(
        //     &ctx, 
        //     vk::Format::R8G8B8A8_SRGB, 
        //     width, 
        //     height, 
        //     &pixels
        // )?;

        // let index = bindless.alloc_texture(&ctx.device, texture.image_view.raw);
        // texture.bindless_index = Some(index);

        // let vertices = vec![
        //     TextureVertex { pos: [0.0, 0.2, 0.0],  uv: [0.5, 0.0] },
        //     TextureVertex { pos: [-0.2, -0.2, 0.0], uv: [0.0, 1.0] },
        //     TextureVertex { pos: [0.2, -0.2, 0.0],  uv: [1.0, 1.0] }
        // ];

        // let mut vertex_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
        //     .size((size_of::<Vertex>() * vertices.len()) as u64)
        //     .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
        //     .build()?;

        // vertex_buffer.upload_data(&vertices)?;

        // let indices = vec![0, 1, 2u32];
        
        // let mut index_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
        //     .size((size_of::<u32>() * indices.len()) as u64)
        //     .usage(vk::BufferUsageFlags::INDEX_BUFFER)
        //     .build()?;

        // index_buffer.upload_data(&indices)?;

        // let ui_mesh = Mesh {
        //     vertex_buffer,
        //     index_buffer,
        //     index_count: 3
        // };

        let pbr_model = load_gltf(&ctx, &bindless, open_vfs(r"assets\meshes\flighthelmet\scene.gltf")).unwrap();

        let descriptor_pool = DescriptorPoolBuilder::new(&ctx.device)
            .max_sets(30)
            .pool_sizes(&[
                vk::DescriptorPoolSize::default()
                    .descriptor_count(5)
                    .ty(vk::DescriptorType::SAMPLED_IMAGE),
                vk::DescriptorPoolSize::default()
                    .descriptor_count(5)
                    .ty(vk::DescriptorType::STORAGE_BUFFER),
                vk::DescriptorPoolSize::default()
                    .descriptor_count(5)
                    .ty(vk::DescriptorType::STORAGE_IMAGE),
                vk::DescriptorPoolSize::default()
                    .descriptor_count(15)
                    .ty(vk::DescriptorType::UNIFORM_BUFFER),
            ])
            .build()?;

        let set_layout = DescriptorSetLayoutBuilder::new(&ctx.device)
            .bindings(vec![
                vk::DescriptorSetLayoutBinding::default()
                    .stage_flags(vk::ShaderStageFlags::VERTEX)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .descriptor_count(1)
            ])
            .build()?;

        let set = descriptor_pool.create_descriptor_set(&ctx.device, &[set_layout.raw])?[0];

        unsafe {

            let buffer_info = vk::DescriptorBufferInfo::default()   
                .buffer(pbr_model.ssbo.as_ref().unwrap().raw)
                .offset(0)
                .range(vk::WHOLE_SIZE);

            let write = vk::WriteDescriptorSet::default()
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .dst_set(set)
                .dst_binding(0)
                .dst_array_element(0)
                .buffer_info(std::slice::from_ref(&buffer_info));

            ctx.device.update_descriptor_sets(&[write], &[]);
        }

        let camera = Camera::new(&ctx)?;

        let camera_set_layout = DescriptorSetLayoutBuilder::new(&ctx.device)
            .bindings(vec![
                vk::DescriptorSetLayoutBinding::default()
                    .stage_flags(vk::ShaderStageFlags::VERTEX)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(1)
            ])
            .build()?;

        let camera_set = descriptor_pool.create_descriptor_set(&ctx.device, &[camera_set_layout.raw])?[0];

        unsafe {

            let buffer_info = vk::DescriptorBufferInfo::default()   
                .buffer(camera.buffer.raw)
                .offset(0)
                .range(vk::WHOLE_SIZE);

            let write = vk::WriteDescriptorSet::default()
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .dst_set(camera_set)
                .dst_binding(0)
                .dst_array_element(0)
                .buffer_info(std::slice::from_ref(&buffer_info));

            ctx.device.update_descriptor_sets(&[write], &[]);
        }

        let sky_renderer = SkyRenderer::new(&ctx, &descriptor_pool)?;
        let simple_ui_renderer = SimpleUiRenderer::new(&ctx, &bindless, &sky_renderer.sampeld_layout)?;

        Ok(Self { 
            set,
            last_time: Instant::now(),
            phys_engine: PhysicsEngine::new(),
            egui_renderer: EguiRenderer::new(&ctx, window),
            render_loop: RenderLoop::new(&ctx),
            camera,
            
            pbr_renderer: PbrRenderer::new(&ctx, &bindless, &descriptor_pool, &set_layout, &camera_set_layout)?,
            simple_ui_renderer,
            sky_renderer,
            //simple_renderer: SimpleRenderer::new(&ctx)?,
            pbr_model,
            set_layout_storage: set_layout,
            pool: descriptor_pool,
            //texture,
            //simple_mesh,
            //ui_mesh,
            camera_set_layout,
            camera_set,
            ctx,
            bindless,
        })
    }
    
    pub fn handle_event(&mut self, winit_window: &Window, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {

        let _ = self.egui_renderer.egui_winit.on_window_event(winit_window, &event);

        self.camera.handle_event(&event);

        match event {
            WindowEvent::Resized(size) => {
                let (width, height) = (size.width, size.height);
                self.ctx.resize(width, height).expect("Error resize window");
            },
            WindowEvent::RedrawRequested => {

                let dt = self.last_time.elapsed().as_secs_f32();
                self.egui_renderer.frame_time = self.last_time.elapsed().as_millis() as f32;
                self.last_time = std::time::Instant::now();

                let resolution = self.ctx.resolution();
                self.camera.update(dt, resolution.width as f32 / resolution.height as f32);

                self.render_loop.draw(&self.ctx, |device, window, cmd, image_index| {

                    transition_image_layout(device, cmd,
                        self.sky_renderer.cube_map.image.raw,
                        vk::ImageLayout::UNDEFINED,
                        vk::ImageLayout::GENERAL,
                    );

                    self.sky_renderer.dispatch_sky(device, cmd);

                    transition_image_layout(device, cmd,
                        self.sky_renderer.cube_map.image.raw,
                        vk::ImageLayout::GENERAL,
                        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    );

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

                    self.simple_ui_renderer.draw_fullscreen(
                        device, 
                        window, 
                        &self.camera,
                        &self.bindless,
                        self.sky_renderer.sampled_cude_set,
                        cmd, 
                    );

                    self.pbr_renderer.draw_mesh(
                        &self.ctx.device, 
                        window, 
                        image_index, 
                        &self.bindless, 
                        self.set,
                        self.camera_set,
                        cmd, 
                        &self.pbr_model
                    );

                    self.egui_renderer.draw(
                        cmd, 
                        winit_window,
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

fn transition_image_layout(
    device: &GraphicsDevice,
    cmd: vk::CommandBuffer,
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
        .old_layout(old_layout)        // ← теперь используем параметры
        .new_layout(new_layout)        // ←
        .src_access_mask(src_access)   // ←
        .dst_access_mask(dst_access)   // ←
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
        device.cmd_pipeline_barrier(
            cmd,
            src_stage,   // ←
            dst_stage,   // ←
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        )
    };
}

impl Drop for GameRenderer {
    fn drop(&mut self) {
        
    }
}