use std::sync::Arc;
use ash::vk;
use parking_lot::RwLockWriteGuard;

use crate::{Bindless, core::{AttributeDescriptions, BindingDescriptions, GpuBuffer, GraphicsPipeline, GraphicsPipelineBuilder, PipelineLayout, PipelineLayoutBuilder, ShaderBuilder, TextureVertex, Vertex, VulkanResult, load_spv}, render_context::{GraphicsDevice, RenderContext, WindowManager}};

pub struct SimpleUiRenderer {
    layout: PipelineLayout,
    pipeline: GraphicsPipeline
}

impl SimpleUiRenderer {
    pub fn new(ctx: &Arc<RenderContext>, bindless: &Bindless) -> VulkanResult<Self> {

        let resolution = ctx.resolution();
        let color_blend = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .blend_enable(false);

        let bytecode = load_spv(r"assets\shaders\spv\raster_ui_vs.spv").expect("Path not found or SPIR-V bytecode not valid");
        let vertex = ShaderBuilder::new(&ctx.device)
            .bytecode(&bytecode)
            .build()?;

        let bytecode = load_spv(r"assets\shaders\spv\raster_ui_ps.spv").expect("Path not found or SPIR-V bytecode not valid");
        let fragment = ShaderBuilder::new(&ctx.device)
            .bytecode(&bytecode)
            .build()?;

        let layout = PipelineLayoutBuilder::new(&ctx.device)
            .set_layouts(vec![
                bindless.set_layout.raw
            ])
            .push_constant(vec![vk::PushConstantRange::default()
                .offset(0)
                .size(128)
                .stage_flags(
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                )])
            .build()?;

        let binds = TextureVertex::bind_desc();
        let attrs = TextureVertex::attr_desc();

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&binds)
            .vertex_attribute_descriptions(&attrs);

        let pipeline = GraphicsPipelineBuilder::new(&ctx.device)
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
            pipeline,
            layout
        })
    }

    pub fn draw_ui(&self, device: &GraphicsDevice, window: &RwLockWriteGuard<'_, WindowManager>, bindless: &Bindless, image_index: u32, cmd: vk::CommandBuffer, vertex_buffer: &GpuBuffer, index_buffer: &GpuBuffer, index_count: u32) {

        unsafe {

            device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline.raw);

            let viewport = vk::Viewport::default()
                .x(0.0)
                .y(0.0)
                .width(window.resolution.width as f32)
                .height(window.resolution.height as f32)
                .min_depth(0.0)
                .max_depth(1.0);

            let viewports = vec![viewport];
            device.cmd_set_viewport(cmd, 0, &viewports);

            let scissor = vk::Rect2D::default()
                .offset(vk::Offset2D { x: 0, y: 0 })
                .extent(window.resolution);

            let scissors = vec![scissor];
            device.cmd_set_scissor(cmd, 0, &scissors);

            device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::GRAPHICS, self.layout.raw, 0, &[bindless.set], &[]);
            device.cmd_bind_vertex_buffers(cmd, 0, &[vertex_buffer.raw], &[0]);
            device.cmd_bind_index_buffer(cmd, index_buffer.raw, 0, vk::IndexType::UINT32);
            device.cmd_draw_indexed(cmd, index_count, 1, 0, 0, 0);

        }
    }

    pub fn destroy(&self, ctx: &Arc<RenderContext>) {
        unsafe {
            ctx.device.destroy_pipeline(self.pipeline.raw, None);
            ctx.device.destroy_pipeline_layout(self.layout.raw, None);
        }
    }
}