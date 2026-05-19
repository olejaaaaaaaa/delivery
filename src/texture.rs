use std::sync::Arc;

use ash::vk;

use crate::{core::{CommandPoolBuilder, GpuBufferBuilder, Image, ImageBuilder, ImageView, ImageViewBuilder, VulkanResult}, render_context::RenderContext};

pub struct Texture {
    pub bindless_index: Option<u32>,
    pub image: Image,
    pub image_view: ImageView
}

impl Texture {
    pub fn new(ctx: &Arc<RenderContext>, format: vk::Format, width: u32, height: u32, pixels: &[u8]) -> VulkanResult<Self> {

        let image = ImageBuilder::new(&ctx.device)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .array_layers(1)
            .format(format)
            .image_type(vk::ImageType::TYPE_2D)
            .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST)
            .build()?;

        let buffer_size = size_of::<u8>() * pixels.len();

        let mut staging_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
            .size(buffer_size as u64)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .build()?;

        staging_buffer.upload_data(pixels)?;

        let transfer_queue = ctx.device.queue_pool.transfer().unwrap();
        let queue = transfer_queue.lock().unwrap();

        let pool = CommandPoolBuilder::reset(&ctx.device).family(queue.family_index).build()?;
        let cmd = pool.allocate_cmd_buffers(&ctx.device, vk::CommandBufferLevel::PRIMARY, 1)?[0];

        let barrier = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .image(image.raw)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            ctx.device.begin_command_buffer(cmd, &begin_info);
        }

        unsafe {
            ctx.device.cmd_pipeline_barrier(
                cmd,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );
        }

        let region = vk::BufferImageCopy::default()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            })
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vk::Extent3D {
                width: width,
                height: height,
                depth: 1,
            });

        unsafe {
            ctx.device.cmd_copy_buffer_to_image(
                cmd,
                staging_buffer.raw,
                image.raw,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
        }

        let barrier2 = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image.raw)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        unsafe {
            ctx.device.cmd_pipeline_barrier(
                cmd,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier2],
            );
        }

        unsafe {
            ctx.device.end_command_buffer(cmd);
        }

        let binding = [cmd];
        let submit_info = vk::SubmitInfo::default().command_buffers(&binding);
        unsafe {
            let _ = ctx
                .device
                .queue_submit(queue.raw, &[submit_info], vk::Fence::null());
            let _ = ctx.device.queue_wait_idle(queue.raw);
        }

        let image_view = ImageViewBuilder::new(&ctx.device)
            .components(vk::ComponentMapping::default())
            .format(format)
            .image(image.raw)
            .view_type(vk::ImageViewType::TYPE_2D)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_array_layer(0)
                    .layer_count(1)
                    .base_mip_level(0)
                    .level_count(1),
            )
            .build()?;

        Ok(Self {
            bindless_index: None,
            image,
            image_view
        })
    }

    pub fn cube(ctx: &Arc<RenderContext>, format: vk::Format, width: u32, height: u32) -> VulkanResult<Self> {

        let image = ImageBuilder::new(&ctx.device)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .array_layers(6)
            .format(format)
            .flags(vk::ImageCreateFlags::CUBE_COMPATIBLE)
            .image_type(vk::ImageType::TYPE_2D)
            .usage(vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED)
            .build()?;

        let image_view = ImageViewBuilder::new(&ctx.device)
            .components(vk::ComponentMapping::default())
            .format(format)
            .image(image.raw)
            .view_type(vk::ImageViewType::TYPE_2D_ARRAY)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_array_layer(0)
                    .layer_count(6)
                    .base_mip_level(0)
                    .level_count(1),
            )
            .build()?;

        Ok(Self {
            bindless_index: None,
            image,
            image_view
        })
    }
}