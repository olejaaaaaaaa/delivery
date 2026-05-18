use std::{sync::Arc, time::Instant};
use ash::vk::{self, Format};
use bytemuck::{Pod, Zeroable};
use parking_lot::RwLockWriteGuard;

use crate::{Texture, core::{AttributeDescriptions, BindingDescriptions, ComputePipeline, ComputePipelineBuilder, DescriptorPool, DescriptorSetLayout, DescriptorSetLayoutBuilder, GpuBuffer, GraphicsPipeline, GraphicsPipelineBuilder, PipelineLayout, PipelineLayoutBuilder, ShaderBuilder, Vertex, VulkanResult, load_spv}, open_vfs, render_context::{GraphicsDevice, RenderContext, WindowManager}, renderer::render_loop::FrameContext};

pub struct SkyRenderer {
    pipeline: ComputePipeline,
    layout: PipelineLayout,
    pub time: Instant,
    pub sky_rendered_once: bool,
    pub compute_layout: DescriptorSetLayout,
    pub sampled_layout: DescriptorSetLayout,
    pub compute_cube_set: vk::DescriptorSet,
    pub sampled_cude_set: vk::DescriptorSet,
    pub cube_map: Texture,
}

impl SkyRenderer {
    pub fn new(ctx: &Arc<RenderContext>, pool: &DescriptorPool) -> VulkanResult<Self> {

        let bytecode = load_spv(open_vfs("atmosphere.spv")).expect("Path not found or SPIR-V bytecode not valid");
        let shader = ShaderBuilder::new(&ctx.device)
            .bytecode(&bytecode)
            .build()?;

        let compute_set_layout = DescriptorSetLayoutBuilder::new(&ctx.device)
            .bindings(vec![
                vk::DescriptorSetLayoutBinding::default()
                    .binding(0)
                    .stage_flags(vk::ShaderStageFlags::COMPUTE)
                    .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                    .descriptor_count(1),
            ])
            .build()?;

        let fragment_set_layout = DescriptorSetLayoutBuilder::new(&ctx.device)
            .bindings(vec![
                vk::DescriptorSetLayoutBinding::default()
                    .binding(0)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                    .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                    .descriptor_count(1),
            ])
            .build()?;

        let sets = pool.create_descriptor_set(
            &ctx.device, 
            &[compute_set_layout.raw, fragment_set_layout.raw]
        )?;

        let (compute_set, fragment_set) = (sets[0], sets[1]);
        let cube_map = Texture::cube(ctx, vk::Format::R32G32B32A32_SFLOAT, 512, 512)?;

        unsafe {

            let compute_image_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::GENERAL)
                .image_view(cube_map.image_view.raw);

            let fragment_image_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(cube_map.image_view.raw);

            ctx.device.update_descriptor_sets(&[
                vk::WriteDescriptorSet::default()
                    .dst_set(compute_set)
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                    .image_info(std::slice::from_ref(&compute_image_info)),
                vk::WriteDescriptorSet::default()
                    .dst_set(fragment_set)
                    .dst_binding(0)
                    .descriptor_count(1)
                    .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                    .image_info(std::slice::from_ref(&fragment_image_info)),
            ], &[]);
        }

        let layout = PipelineLayoutBuilder::new(&ctx.device)
            .set_layouts(vec![
                compute_set_layout.raw
            ])
            .push_constant(vec![vk::PushConstantRange::default()
                .offset(0)
                .size(128)
                .stage_flags(
                    vk::ShaderStageFlags::COMPUTE,
                )])
            .build()?;

        let pipeline = ComputePipelineBuilder::new(&ctx.device)
            .pipeline_layout(layout.raw)
            .shader(shader.raw)
            .build()?;

        Ok(Self {
            time: Instant::now(),
            sky_rendered_once: false,
            cube_map,
            pipeline,
            sampled_layout: fragment_set_layout,
            compute_layout: compute_set_layout,
            compute_cube_set: compute_set,
            sampled_cude_set: fragment_set,
            layout
        })
    }

    pub fn dispatch_sky(&self, frame: &FrameContext ) {
        unsafe {
            frame.device.cmd_bind_pipeline(frame.cmd, vk::PipelineBindPoint::COMPUTE, self.pipeline.raw);
            frame.device.cmd_bind_descriptor_sets(
            frame.cmd,
            vk::PipelineBindPoint::COMPUTE,
                self.layout.raw,
                0,
                &[self.compute_cube_set],
                &[],
            );

            #[repr(C)]
            #[derive(Clone, Copy, Debug, Pod, Zeroable, Default)]
            struct FrameConstants {
                sky_ambient: [f32; 4],
                sun_color_multiplier: [f32; 4],
                sun_direction: [f32; 3], 
                pre_exposure: f32
            }

            let push = FrameConstants {
                sky_ambient: [0.0, 0.0, 0.0, 0.0],
                sun_color_multiplier: [1.0, 0.95, 0.8, 0.0],
                pre_exposure: 1.0 / 2.0, 
                sun_direction: [
                    0.0, 
                    1.0, 
                    0.0
                ],
            };

            frame.device.cmd_push_constants(
                frame.cmd,
                self.layout.raw,
                vk::ShaderStageFlags::COMPUTE,
                0,
                bytemuck::bytes_of(&push),
            );

            frame.device.cmd_dispatch(frame.cmd, 512/8, 512/8, 6);
        }
    }

    pub fn destroy(&self, ctx: &Arc<RenderContext>) {
        unsafe {
            ctx.device.destroy_pipeline(self.pipeline.raw, None);
            ctx.device.destroy_pipeline_layout(self.layout.raw, None);
        }
    }
}