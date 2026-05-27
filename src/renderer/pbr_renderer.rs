use std::sync::Arc;
use ash::vk;
use bytemuck::{Pod, Zeroable};
use parking_lot::RwLockWriteGuard;

use crate::{Bindless, Camera, Car, GltfModel, Resources, SetLayoutDesc, SkyRenderer, Track, core::{AttributeDescriptions, BindingDescriptions, DescriptorPool, DescriptorSetLayout, DescriptorSetLayoutBuilder, GpuBuffer, GraphicsPipeline, GraphicsPipelineBuilder, PbrVertex, PipelineLayout, PipelineLayoutBuilder, ShaderBuilder, Vertex, VulkanResult, load_spv}, open_vfs, render_context::{GraphicsDevice, RenderContext, WindowManager}, renderer::render_loop::FrameContext};

pub struct PbrRenderer {
    layout: PipelineLayout,
    pipeline: GraphicsPipeline
}

impl PbrRenderer {
    pub fn new(ctx: &Arc<RenderContext>, res: &Arc<Resources>) -> VulkanResult<Self> {

        let resolution = ctx.resolution();
        let color_blend = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .blend_enable(false);

        let bytecode = load_spv(open_vfs("shader://raster_pbr_vs.spv")).expect("Path not found or SPIR-V bytecode not valid");
        let vertex = ShaderBuilder::new(&ctx.device)
            .bytecode(&bytecode)
            .build()?;

        let bytecode = load_spv(open_vfs("shader://raster_pbr_ps.spv")).expect("Path not found or SPIR-V bytecode not valid");
        let fragment = ShaderBuilder::new(&ctx.device)
            .bytecode(&bytecode)
            .build()?;

        let mut cache = res.set_layout.lock();
        let cube_set_layout = cache.get_or_create(&ctx.device, SetLayoutDesc {
            bindings: vec![
                vk::DescriptorSetLayoutBinding::default()
                    .binding(0)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                    .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                    .descriptor_count(1)
            ]
        })?;

        let camera_set_layout = DescriptorSetLayoutBuilder::new(&ctx.device)
            .bindings(&[
                vk::DescriptorSetLayoutBinding::default()
                    .stage_flags(vk::ShaderStageFlags::VERTEX)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(1)
            ])
            .build()?;

        let transforms_set_layout = DescriptorSetLayoutBuilder::new(&ctx.device)
            .bindings(&[
                vk::DescriptorSetLayoutBinding::default()
                    .stage_flags(vk::ShaderStageFlags::VERTEX)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .descriptor_count(1)
            ])
            .build()?;

        let layout = PipelineLayoutBuilder::new(&ctx.device)
            .set_layouts(&[
                res.bindless.set_layout.raw, 
                transforms_set_layout.raw, 
                camera_set_layout.raw,
                cube_set_layout.raw
            ])
            .push_constant(&[vk::PushConstantRange::default()
                .offset(0)
                .size(128)
                .stage_flags(
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                )])
            .build()?;

        let binds = PbrVertex::bind_desc();
        let attrs = PbrVertex::attr_desc();

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&binds)
            .vertex_attribute_descriptions(&attrs);

        let dynamic_rendering = vk::PipelineRenderingCreateInfoKHR::default()
            .color_attachment_formats(std::slice::from_ref(&vk::Format::R8G8B8A8_SRGB))
            .depth_attachment_format(vk::Format::D32_SFLOAT);

        let pipeline = GraphicsPipelineBuilder::new(&ctx.device)
            .dynamic_rendering(dynamic_rendering)
            .vertex_shader(vertex.raw)
            .fragment_shader(fragment.raw)
            //.render_pass(ctx.window.read().render_pass.raw)
            .pipeline_layout(layout.raw)
            .depth_stencil(
                vk::PipelineDepthStencilStateCreateInfo::default()
                    .depth_test_enable(true)
                    .depth_write_enable(true)
                    .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
                    .depth_bounds_test_enable(false)
                    .stencil_test_enable(false),
            )
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

    pub fn draw_mesh(&self, frame: &FrameContext<'_>, bindless: &Bindless, sky: &SkyRenderer, camera: &Camera, car: &Car, track: &Track) {

        if let Some(model) = &track.model {

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

                frame.device.cmd_bind_descriptor_sets(
                    frame.cmd, 
                    vk::PipelineBindPoint::GRAPHICS, 
                    self.layout.raw, 
                    0, 
                    &[bindless.set, track.set.unwrap(), camera.set, sky.sampled_cude_set], 
                    &[]
                );

                #[repr(C)]
                #[derive(Clone, Copy, Debug, Pod, Zeroable, Default)]
                struct Push {
                    tex_idx: [u32; 8],
                    user_data: [f32; 24]
                }

                let mut push = Push::default();

                for (index, (mesh, material)) in model.meshes.iter().enumerate() {

                    push.user_data[0] = 0.0;
                    push.user_data[1] = 15.0;
                    push.user_data[2] = 15.0;
                    push.user_data[3] = index as f32;

                    push.tex_idx[0] = material.diffuse_map;
                    push.tex_idx[1] = material.metallic_roughness_map;
                    push.tex_idx[2] = material.occlusion_map;
                    push.tex_idx[3] = material.normal_map;

                    frame.device.cmd_push_constants(
                        frame.cmd,
                        self.layout.raw,
                        vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                        0,
                        bytemuck::bytes_of(&push),
                    );

                    frame.device.cmd_bind_vertex_buffers(frame.cmd, 0, &[mesh.vertex_buffer.raw], &[0]);
                    frame.device.cmd_bind_index_buffer(frame.cmd, mesh.index_buffer.raw, 0, vk::IndexType::UINT32);
                    frame.device.cmd_draw_indexed(frame.cmd, mesh.index_count, 1, 0, 0, 0);
                }

                if let Some(model) = &car.car_body {
                         
                    frame.device.cmd_bind_descriptor_sets(
                        frame.cmd, 
                        vk::PipelineBindPoint::GRAPHICS, 
                        self.layout.raw, 
                        0, 
                        &[bindless.set, car.car_set.as_ref().unwrap()[frame.image_index as usize], camera.set], 
                        &[]
                    );

                    let mut push = Push::default();

                    for (index, (mesh, material)) in model.meshes.iter().enumerate() {

                        push.user_data[0] = 0.0;
                        push.user_data[1] = 15.0;
                        push.user_data[2] = 15.0;
                        push.user_data[3] = index as f32;

                        push.tex_idx[0] = material.diffuse_map;
                        push.tex_idx[1] = material.metallic_roughness_map;
                        push.tex_idx[2] = material.occlusion_map;
                        push.tex_idx[3] = material.normal_map;

                        frame.device.cmd_push_constants(
                            frame.cmd,
                            self.layout.raw,
                            vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                            0,
                            bytemuck::bytes_of(&push),
                        );

                        frame.device.cmd_bind_vertex_buffers(frame.cmd, 0, &[mesh.vertex_buffer.raw], &[0]);
                        frame.device.cmd_bind_index_buffer(frame.cmd, mesh.index_buffer.raw, 0, vk::IndexType::UINT32);
                        frame.device.cmd_draw_indexed(frame.cmd, mesh.index_count, 1, 0, 0, 0);
                    }
                }
            }
        }

    }

    pub fn destroy(&self, ctx: &Arc<RenderContext>) {
        unsafe {
            ctx.device.destroy_pipeline(self.pipeline.raw, None);
            ctx.device.destroy_pipeline_layout(self.layout.raw, None);
        }
    }
}