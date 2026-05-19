use std::{sync::Arc, time::Instant};
use ash::vk;
use bytemuck::{Pod, Zeroable};
use parking_lot::RwLockWriteGuard;
use winit::window::Window;

use crate::{Bindless, Camera, SkyRenderer, Texture, core::{AttributeDescriptions, BindingDescriptions, DescriptorSetLayout, GpuBuffer, GpuBufferBuilder, GraphicsPipeline, GraphicsPipelineBuilder, PipelineLayout, PipelineLayoutBuilder, ShaderBuilder, TextureVertex, Vertex, VulkanResult, load_spv}, open_vfs, render_context::{GraphicsDevice, RenderContext, WindowManager}, renderer::render_loop::FrameContext};

pub struct LoaderRenderer {
    time: Instant,
    text_texture: Texture,
    text_mesh: GpuBuffer,
    layout: PipelineLayout,
    pipeline: GraphicsPipeline
}

impl LoaderRenderer {
    pub fn new(ctx: &Arc<RenderContext>, bindless: &Bindless) -> VulkanResult<Self> {

        let texture = image::open(open_vfs(r"assets\textures\text.png")).unwrap();
        let rgba8 = texture.as_rgba8().unwrap();
        let (width, height) = (rgba8.width(), rgba8.height());
        let pixels = rgba8.clone().into_raw();

        let mut texture = Texture::new(
            ctx, 
            vk::Format::R8G8B8A8_UNORM, 
            width, 
            height, 
            &pixels
        )?;

        texture.bindless_index = Some(bindless.alloc_texture(&ctx.device, texture.image_view.raw));

        let tex_width = width as f32;
        let tex_height = height as f32;

        let aspect = tex_width / tex_height;

        let offset_x = 0.0;
        let offset_y = 0.5;

        let scale_y = 0.05;
        let scale_x = scale_y * aspect * 0.5;

        let tex_mesh = vec![
            TextureVertex { pos: [-scale_x + offset_x,  scale_y + offset_y, 0.0], uv: [0.0, 1.0] },
            TextureVertex { pos: [ scale_x + offset_x,  scale_y + offset_y, 0.0], uv: [1.0, 1.0] },
            TextureVertex { pos: [-scale_x + offset_x, -scale_y + offset_y, 0.0], uv: [0.0, 0.0] },

            TextureVertex { pos: [ scale_x + offset_x,  scale_y + offset_y, 0.0], uv: [1.0, 1.0] },
            TextureVertex { pos: [ scale_x + offset_x, -scale_y + offset_y, 0.0], uv: [1.0, 0.0] },
            TextureVertex { pos: [-scale_x + offset_x, -scale_y + offset_y, 0.0], uv: [0.0, 0.0] },
        ];

        // let tex_mesh = vec![
        //     TextureVertex { pos: [-1.0,  1.0, 0.0], uv: [0.0, 1.0] },  // верхний левый
        //     TextureVertex { pos: [ 1.0,  1.0, 0.0], uv: [1.0, 1.0] },  // верхний правый
        //     TextureVertex { pos: [-1.0, -1.0, 0.0], uv: [0.0, 0.0] },  // нижний левый

        //     TextureVertex { pos: [ 1.0,  1.0, 0.0], uv: [1.0, 1.0] },  // верхний правый
        //     TextureVertex { pos: [ 1.0, -1.0, 0.0], uv: [1.0, 0.0] },  // нижний правый
        //     TextureVertex { pos: [-1.0, -1.0, 0.0], uv: [0.0, 0.0] },  // нижний левый
        // ];

        let mut text_mesh = GpuBufferBuilder::cpu_only(&ctx.device)
            .size((size_of::<TextureVertex>() * tex_mesh.len()) as u64)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .build()?;

        text_mesh.upload_data(&tex_mesh).unwrap();

        let resolution = ctx.resolution();
        let color_blend = vk::PipelineColorBlendAttachmentState::default()
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(vk::ColorComponentFlags::RGBA);

        let bytecode = load_spv(open_vfs("raster_loader_vs.spv")).expect("Path not found or SPIR-V bytecode not valid");
        let vertex = ShaderBuilder::new(&ctx.device)
            .bytecode(&bytecode)
            .build()?;

        let bytecode = load_spv(open_vfs("raster_loader_ps.spv")).expect("Path not found or SPIR-V bytecode not valid");
        let fragment = ShaderBuilder::new(&ctx.device)
            .bytecode(&bytecode)
            .build()?;

        let layout = PipelineLayoutBuilder::new(&ctx.device)
            .set_layouts(vec![
                bindless.set_layout.raw,
            ])
            .push_constant(vec![vk::PushConstantRange::default()
                .offset(0)
                .size(128)
                .stage_flags(
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                )])
            .build()?;

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(false)
            .depth_write_enable(false)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
            .depth_bounds_test_enable(false)
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0)
            .stencil_test_enable(false);

        let attrs = TextureVertex::attr_desc();
        let binds = TextureVertex::bind_desc();

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_attribute_descriptions(&attrs)
            .vertex_binding_descriptions(&binds);

        let pipeline = GraphicsPipelineBuilder::new(&ctx.device)
            .depth_stencil(depth_stencil)
            .vertex_shader(vertex.raw)
            .fragment_shader(fragment.raw)
            .render_pass(ctx.window.read().render_pass.raw)
            .pipeline_layout(layout.raw)
            .viewport(vec![vk::Viewport::default()
                .x(0.0)
                .y(0.0)
                .width(resolution.width as f32)
                .height(resolution.height as f32)
                .min_depth(0.0)
                .max_depth(1.0)])
            .scissors(vec![vk::Rect2D::default()
                .offset(vk::Offset2D { x: 0, y: 0 })
                .extent(resolution)])
            .input_assembly(
                vk::PipelineInputAssemblyStateCreateInfo::default()
                    .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                    .primitive_restart_enable(false),
            )
            .rasterization(
                vk::PipelineRasterizationStateCreateInfo::default()
                    .depth_clamp_enable(false)
                    .rasterizer_discard_enable(false)
                    .polygon_mode(vk::PolygonMode::FILL)
                    .line_width(1.0)
                    .cull_mode(vk::CullModeFlags::NONE)
                    .front_face(vk::FrontFace::CLOCKWISE)
                    .depth_bias_enable(false),
            )
            .multisampling(
                vk::PipelineMultisampleStateCreateInfo::default()
                    .sample_shading_enable(false)
                    .rasterization_samples(vk::SampleCountFlags::TYPE_1),
            )
            .color_blending(
                vk::PipelineColorBlendStateCreateInfo::default()
                    .logic_op_enable(false)
                    .logic_op(vk::LogicOp::COPY)
                    .attachments(&[color_blend]),
            )
            .dynamic_state(vec![
                vk::DynamicState::VIEWPORT,
                vk::DynamicState::SCISSOR
            ])
            .vertex_input_info(vertex_input_info)
            .build()?;

        Ok(Self {
            time: Instant::now(),
            text_mesh,
            text_texture: texture,
            pipeline,
            layout
        })
    }

    pub fn draw_fullscreen(&self, frame: &FrameContext<'_>, bindless: &Bindless, window: &Window) {

        unsafe {

            frame.device.cmd_bind_pipeline(frame.cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline.raw);

            let viewport = vk::Viewport::default()
                .x(0.0)
                .y(0.0)
                .width(frame.window.resolution.width as f32)
                .height(frame.window.resolution.height as f32)
                .min_depth(0.0)
                .max_depth(1.0);

            let viewports = vec![viewport];
            frame.device.cmd_set_viewport(frame.cmd, 0, &viewports);

            let scissor = vk::Rect2D::default()
                .offset(vk::Offset2D { x: 0, y: 0 })
                .extent(frame.window.resolution);

            let scissors = vec![scissor];
            frame.device.cmd_set_scissor(frame.cmd, 0, &scissors);
            frame.device.cmd_bind_descriptor_sets(frame.cmd, vk::PipelineBindPoint::GRAPHICS, self.layout.raw, 0, &[bindless.set], &[]);

            #[repr(C)]
            #[derive(Clone, Copy, Pod, Zeroable)]
            struct PushConstants {
                text_idx: u32,
                // pad: u32,
                // speed: f32,
                // time: f32,
                // resolution: [u32; 2]
            }

            let mut push = PushConstants::zeroed();
            push.text_idx = self.text_texture.bindless_index.unwrap();
            // push.speed = 0.5;
            // push.time = self.time.elapsed().as_secs_f32();
            // push.resolution = [window.inner_size().width, window.inner_size().height];

            frame.device.cmd_push_constants(
                frame.cmd,
                self.layout.raw,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::bytes_of(&push),
            );

            frame.device.cmd_bind_vertex_buffers(
                frame.cmd, 
                0, 
                &[self.text_mesh.raw], 
                &[0]
            );

            frame.device.cmd_draw(frame.cmd, 6, 1, 0, 0);

        }
    }

    pub fn destroy(&self, ctx: &Arc<RenderContext>) {
        unsafe {
            ctx.device.destroy_pipeline(self.pipeline.raw, None);
            ctx.device.destroy_pipeline_layout(self.layout.raw, None);
        }
    }
}