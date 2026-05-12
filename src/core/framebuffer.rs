use ash::vk::{self};
use tracing::debug;

use super::device::Device;
use super::{VulkanError, VulkanResult};
use crate::core::debug;

pub struct FrameBuffer {
    pub raw: vk::Framebuffer,
}

impl FrameBuffer {
    pub fn destroy(&self, device: &Device) {
        unsafe { device.destroy_framebuffer(self.raw, None) };
        debug!(
            handle = ?self.raw,
            "FrameBuffer destroyed"
        );
    }
}

pub struct FrameBufferBuilder<'a> {
    device: &'a Device,
    extent: Option<vk::Extent2D>,
    attachments: Option<&'a [vk::ImageView]>,
    layers: Option<u32>,
    render_pass: Option<vk::RenderPass>,
}

impl<'a> FrameBufferBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        FrameBufferBuilder {
            device,
            extent: None,
            attachments: None,
            layers: None,
            render_pass: None,
        }
    }

    pub fn layers(mut self, layers: u32) -> Self {
        self.layers = Some(layers);
        self
    }

    pub fn attachments(mut self, attachments: &'a [vk::ImageView]) -> Self {
        self.attachments = Some(attachments);
        self
    }

    pub fn extent(mut self, extent: vk::Extent2D) -> Self {
        self.extent = Some(extent);
        self
    }

    pub fn render_pass(mut self, render_pass: vk::RenderPass) -> Self {
        self.render_pass = Some(render_pass);
        self
    }

    pub fn build(self) -> VulkanResult<FrameBuffer> {
        let render_pass = self.render_pass.expect("Missing RenderPass");
        let extent = self.extent.expect("Missing Extent");
        let layers = self.layers.expect("Missing Layers");
        let attachments = self.attachments.expect("Missing Attachments");

        #[cfg(debug_assertions)]
        {
            if layers == 0 {
                panic!("Layer must be 1 or less");
            }

            if extent.width == 0 || extent.height == 0 {
                panic!("Extent must have width and height with size less 0")
            }

            if attachments.is_empty() {
                panic!("Attachemnts musth have 1 or less attachments")
            }
        }

        let create_info = vk::FramebufferCreateInfo::default()
            .render_pass(render_pass)
            .layers(layers)
            .width(extent.width)
            .height(extent.height)
            .attachments(attachments);

        let frame_buffer = unsafe {
            profiling::scope!("vkCreateFramebuffer");
            self.device
                .create_framebuffer(&create_info, None)
                .map_err(VulkanError::Unknown)?
        };

        debug!(
            handle = ?frame_buffer,
            extent = ?extent,
            layers = layers,
            attachments = ?attachments,
            render_pass = ?render_pass,
            "FrameBuffer created"
        );

        Ok(FrameBuffer { raw: frame_buffer })
    }
}
