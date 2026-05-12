use ash::vk;
use tracing::debug;

use super::device::Device;
use super::{VulkanError, VulkanResult};

pub struct ImageView {
    pub(crate) raw: vk::ImageView,
}

impl ImageView {
    pub fn destroy(&self, device: &Device) {
        unsafe { device.destroy_image_view(self.raw, None) };
        debug!(
            handle = ?self.raw,
            "ImageView destroyed"
        );
    }
}

pub struct ImageViewBuilder<'a> {
    device: &'a Device,
    components: Option<vk::ComponentMapping>,
    format: Option<vk::Format>,
    image: Option<vk::Image>,
    subresource_range: Option<vk::ImageSubresourceRange>,
    view_type: Option<vk::ImageViewType>,
}

impl<'a> ImageViewBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            device,
            components: None,
            format: None,
            image: None,
            subresource_range: None,
            view_type: None,
        }
    }

    pub fn format(mut self, format: vk::Format) -> Self {
        self.format = Some(format);
        self
    }

    pub fn image(mut self, image: vk::Image) -> Self {
        self.image = Some(image);
        self
    }

    pub fn components(mut self, components: vk::ComponentMapping) -> Self {
        self.components = Some(components);
        self
    }

    pub fn subresource_range(mut self, subresource_range: vk::ImageSubresourceRange) -> Self {
        self.subresource_range = Some(subresource_range);
        self
    }

    pub fn view_type(mut self, view_type: vk::ImageViewType) -> Self {
        self.view_type = Some(view_type);
        self
    }

    pub fn build(self) -> VulkanResult<ImageView> {
        let format = self.format.expect("Missing Format");
        let image = self.image.expect("Missing Image");
        let components = self.components.unwrap_or(vk::ComponentMapping::default());
        let subresource = self.subresource_range.expect("Missing SubResourceRange");
        let view_type = self.view_type.unwrap_or(vk::ImageViewType::TYPE_2D);

        let create_info = vk::ImageViewCreateInfo::default()
            .subresource_range(subresource)
            .view_type(view_type)
            .components(components)
            .format(format)
            .image(image);

        let image_view = unsafe {
            profiling::scope!("vkCreateImageView");
            self.device
                .create_image_view(&create_info, None)
                .map_err(VulkanError::Unknown)?
        };

        Ok(ImageView { raw: image_view })
    }
}
