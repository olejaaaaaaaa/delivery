use ash::vk;
use tracing::info;

use super::GraphicsDevice;
use crate::core::{
    FrameBuffer, FrameBufferBuilder, FrameSync, Image, ImageBuilder, ImageView, ImageViewBuilder,
    RenderPass, Surface, Swapchain, SwapchainBuilder, VulkanResult,
};

/// Manages window-related Vulkan resources (swapchain, framebuffers,
/// etc.) Handles window resizing and frame synchronization
pub struct WindowManager {
    /// Current window/swapchain resolution
    pub(crate) resolution: vk::Extent2D,
    /// Synchronization objects for each frame in flight
    pub(crate) frame_sync: Vec<FrameSync>,
    /// Framebuffers (one per swapchain image)
    pub(crate) frame_buffers: Vec<FrameBuffer>,
    /// Image views for swapchain images
    pub(crate) image_views: Vec<ImageView>,
    /// Index of current frame
    pub(crate) current_frame: usize,
    /// Depth buffer image
    pub(crate) depth_image: Image,
    /// View into the depth buffer
    pub(crate) depth_view: ImageView,
    /// Render pass defining attachment formats and operations
    pub(crate) render_pass: RenderPass,
    /// Window surface for presentation
    pub(crate) surface: Surface,
    /// Swapchain containing presentable images
    pub(crate) swapchain: Swapchain,
}

impl WindowManager {
    /// Recreate swapchain, image views, depth image, framebuffers for
    /// new window size
    pub fn resize(&mut self, device: &GraphicsDevice, width: u32, height: u32) -> VulkanResult<()> {
        profiling::scope!("WindowManager::resize");

        info!("New size: {:?}", (width, height));

        unsafe { device.device_wait_idle().expect("Error wait idle") };

        let caps = self
            .surface
            .get_physical_device_surface_capabilities(*device.phys_dev)?;

        let formats = self
            .surface
            .get_physical_device_surface_formats(*device.phys_dev)?;

        let extent = caps.current_extent;
        let _transforms = caps.current_transform;

        let format_priority = [vk::Format::R8G8B8A8_SRGB];

        let color_space_priority = [vk::ColorSpaceKHR::SRGB_NONLINEAR];

        let mut format = vk::Format::R8G8B8A8_SRGB;
        let mut color_space = vk::ColorSpaceKHR::SRGB_NONLINEAR;

        for (f, c) in format_priority.iter().zip(color_space_priority.iter()) {
            for j in &formats {
                if *f == j.format && *c == j.color_space {
                    format = j.format;
                    color_space = j.color_space;
                }
            }
        }

        let swapchain = SwapchainBuilder::new(device)
            .old_swapchain(self.swapchain.raw)
            .min_image_count(2)
            .surface(&self.surface)
            .present_mode(vk::PresentModeKHR::FIFO)
            .instance(&device.instance)
            .color_space(color_space)
            .extent(extent)
            .format(format)
            .build()?;

        let depth_image = ImageBuilder::new(device)
            .extent(caps.current_extent.into())
            .format(vk::Format::D32_SFLOAT)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .build()?;

        let depth_view = ImageViewBuilder::new(device)
            .format(vk::Format::D32_SFLOAT)
            .image(depth_image.raw)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .view_type(vk::ImageViewType::TYPE_2D)
            .build()?;

        let mut image_views = vec![];

        for i in swapchain.get_swapchain_images()? {
            let image_view = ImageViewBuilder::new(device)
                .format(vk::Format::R8G8B8A8_SRGB)
                .image(i)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .view_type(vk::ImageViewType::TYPE_2D)
                .build()?;
            image_views.push(image_view);
        }

        let mut frame_buffers = vec![];

        for i in &image_views {
            let frame_buffer = FrameBufferBuilder::new(device)
                .render_pass(self.render_pass.raw)
                .attachments(&[i.raw, depth_view.raw])
                .extent(caps.current_extent)
                .layers(1)
                .build()?;

            frame_buffers.push(frame_buffer);
        }

        self.depth_view.destroy(device);
        self.depth_view = depth_view;

        self.depth_image.destroy(device);
        self.depth_image = depth_image;

        for i in &self.image_views {
            i.destroy(device);
        }

        self.image_views = image_views;

        for i in &self.frame_buffers {
            i.destroy(device);
        }

        self.frame_buffers = frame_buffers;

        self.swapchain.destroy();
        self.swapchain = swapchain;

        self.resolution = caps.current_extent;

        Ok(())
    }
}
