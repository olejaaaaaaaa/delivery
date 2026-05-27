use std::{sync::Arc, time::Instant};
use ash::vk;
use bytemuck::{Pod, Zeroable};
use parking_lot::RwLockWriteGuard;
use winit::{event::{DeviceEvent, WindowEvent}, event_loop::ActiveEventLoop, window::{Window, WindowId}};

use crate::{Bindless, Camera, SkyRenderer, Texture, core::{AttributeDescriptions, BindingDescriptions, DescriptorSetLayout, GpuBuffer, GpuBufferBuilder, GraphicsPipeline, GraphicsPipelineBuilder, PipelineLayout, PipelineLayoutBuilder, ShaderBuilder, TextureVertex, Vertex, VulkanResult, load_spv}, open_vfs, render_context::{GraphicsDevice, RenderContext, WindowManager}, renderer::render_loop::FrameContext};

pub struct Loader2Renderer {
    last_mouse_pos: [f32; 2],
    time: Instant,
    layout: PipelineLayout,
    pipeline: GraphicsPipeline
}

impl Loader2Renderer {
    pub fn new(ctx: &Arc<RenderContext>) -> VulkanResult<Self> {

        let resolution = ctx.resolution();
        let color_blend = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .blend_enable(false);

        let bytecode = load_spv(open_vfs("shader://raster_loader2_vs.spv")).expect("Path not found or SPIR-V bytecode not valid");
        let vertex = ShaderBuilder::new(&ctx.device)
            .bytecode(&bytecode)
            .build()?;

        let bytecode = load_spv(open_vfs("shader://raster_loader2_ps.spv")).expect("Path not found or SPIR-V bytecode not valid");
        let fragment = ShaderBuilder::new(&ctx.device)
            .bytecode(&bytecode)
            .build()?;

        let layout = PipelineLayoutBuilder::new(&ctx.device)
            .set_layouts(&[])
            .push_constant(&[vk::PushConstantRange::default()
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

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default();

        let dynamic_rendering = vk::PipelineRenderingCreateInfoKHR::default()
            .color_attachment_formats(std::slice::from_ref(&vk::Format::R8G8B8A8_SRGB))
            .depth_attachment_format(vk::Format::D32_SFLOAT);

        let pipeline = GraphicsPipelineBuilder::new(&ctx.device)
            .dynamic_rendering(dynamic_rendering)
            .depth_stencil(depth_stencil)
            .vertex_shader(vertex.raw)
            .fragment_shader(fragment.raw)
            //.render_pass(ctx.window.read().render_pass.raw)
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
            last_mouse_pos: [0.0, 0.0],
            time: Instant::now(),
            pipeline,
            layout
        })
    }

    pub fn handle_event(&mut self, winit_window: &Window, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { device_id, position } => {
                let width = winit_window.inner_size().width as f32;
                let height = winit_window.inner_size().height as f32;
                let x = (position.x as f32 - 0.5 * width) / height;
                let y = (position.y as f32 - 0.5 * height) / height;
                self.last_mouse_pos = [x, y];
                //println!("mouse pos: {:?}", self.last_mouse_pos);
            }
            _ => {}
        }
    }

    pub fn draw_fullscreen(&self, frame: &FrameContext<'_>, window: &Window) {

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

            #[repr(C)]
            #[derive(Clone, Copy, Pod, Zeroable)]
            struct PushConstants {
                speed: f32,
                time: f32,
                resolution: [f32; 2],
                mouse_pos: [f32; 2]
            }

            let mut push = PushConstants::zeroed();
            push.speed = 0.5;
            push.time = self.time.elapsed().as_secs_f32();
            push.resolution = [window.inner_size().width as f32, window.inner_size().height as f32];
            push.mouse_pos = self.last_mouse_pos;

            frame.device.cmd_push_constants(
                frame.cmd,
                self.layout.raw,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::bytes_of(&push),
            );

            frame.device.cmd_draw(frame.cmd, 3, 1, 0, 0);

        }
    }

    pub fn destroy(&self, ctx: &Arc<RenderContext>) {
        unsafe {
            ctx.device.destroy_pipeline(self.pipeline.raw, None);
            ctx.device.destroy_pipeline_layout(self.layout.raw, None);
        }
    }
}