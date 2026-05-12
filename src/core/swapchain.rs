use ash::vk::{self, ColorSpaceKHR};
use tracing::debug;

use super::device::Device;
use super::{Instance, Surface, VulkanError, VulkanResult};

pub struct Swapchain {
    pub(crate) raw: vk::SwapchainKHR,
    pub(crate) loader: ash::khr::swapchain::Device,
}

pub struct SwapchainBuilder<'a> {
    extent: Option<vk::Extent2D>,
    old_swapchain: Option<vk::SwapchainKHR>,
    format: Option<vk::Format>,
    present_mode: Option<vk::PresentModeKHR>,
    color_space: Option<vk::ColorSpaceKHR>,
    min_image_count: Option<u32>,
    surface: Option<&'a Surface>,
    instance: Option<&'a Instance>,
    device: &'a Device,
}

impl<'a> SwapchainBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        SwapchainBuilder {
            device,
            color_space: None,
            extent: None,
            old_swapchain: None,
            format: None,
            present_mode: None,
            min_image_count: None,
            surface: None,
            instance: None,
        }
    }

    pub fn color_space(mut self, space: ColorSpaceKHR) -> Self {
        self.color_space = Some(space);
        self
    }

    pub fn surface(mut self, surface: &'a Surface) -> Self {
        self.surface = Some(surface);
        self
    }

    pub fn instance(mut self, instance: &'a Instance) -> Self {
        self.instance = Some(instance);
        self
    }

    pub fn present_mode(mut self, mode: vk::PresentModeKHR) -> Self {
        self.present_mode = Some(mode);
        self
    }

    pub fn old_swapchain(mut self, swapchian: vk::SwapchainKHR) -> Self {
        self.old_swapchain = Some(swapchian);
        self
    }

    pub fn format(mut self, format: vk::Format) -> Self {
        self.format = Some(format);
        self
    }

    pub fn extent(mut self, extent: vk::Extent2D) -> Self {
        self.extent = Some(extent);
        self
    }

    pub fn min_image_count(mut self, count: u32) -> Self {
        self.min_image_count = Some(count);
        self
    }

    pub fn build(self) -> VulkanResult<Swapchain> {
        let instance = self.instance.expect("Missing Instance");
        let surface = self.surface.expect("Missing Surface");
        let extent = self.extent.expect("Missing Extent");
        let present_mode = self.present_mode.expect("Missing Present mode");
        let min_image_count = self.min_image_count.expect("Missing Min image count");
        let format = self.format.expect("Missing Format");
        let color_space = self.color_space.expect("Missing Color space");
        let old_swapchain = self.old_swapchain.unwrap_or(vk::SwapchainKHR::null());

        let swapchain_loader = ash::khr::swapchain::Device::new(&instance.raw, &self.device.raw);

        let create_info = vk::SwapchainCreateInfoKHR::default()
            .clipped(true)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .image_format(format)
            .surface(surface.raw)
            .image_extent(extent)
            .min_image_count(min_image_count)
            .present_mode(present_mode)
            .old_swapchain(old_swapchain)
            .image_color_space(color_space);

        let swapchain = unsafe {
            profiling::scope!("vkCreateSwapchainKHR");
            swapchain_loader
                .create_swapchain(&create_info, None)
                .map_err(|e| VulkanError::Unknown(e))
        }?;

        debug!(
            handle = ?swapchain,
            clipped = true,
            format = ?format,
            min_image_count = ?min_image_count,
            present_mode = ?present_mode,
            color_space = ?color_space,
            old_swapchain = ?old_swapchain,
            extent = ?extent,
            "Swapchain created"
        );

        Ok(Swapchain {
            raw: swapchain,
            loader: swapchain_loader,
        })
    }
}

impl Swapchain {
    pub fn get_swapchain_images(&self) -> VulkanResult<Vec<vk::Image>> {
        unsafe {
            self.loader
                .get_swapchain_images(self.raw)
                .map_err(VulkanError::Unknown)
        }
    }

    pub fn destroy(&self) {
        unsafe {
            profiling::scope!("vkDestroySwapchain");
            self.loader.destroy_swapchain(self.raw, None);
        }
        debug!(
            handle = ?self.raw,
            "Swapchain destroyed"
        )
    }
}
