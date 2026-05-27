use ash::vk;
use tracing::debug;

use super::{Device, VulkanError, VulkanResult};

pub struct PipelineLayout {
    pub(crate) raw: vk::PipelineLayout,
}

pub struct PipelineLayoutBuilder<'a> {
    layouts: Option<&'a [vk::DescriptorSetLayout]>,
    push: Option<&'a [vk::PushConstantRange]>,
    device: &'a Device,
}

impl<'a> PipelineLayoutBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            push: None,
            layouts: None,
            device,
        }
    }

    pub fn set_layouts(mut self, layouts: &'a [vk::DescriptorSetLayout]) -> Self {
        self.layouts = Some(layouts);
        self
    }

    pub fn push_constant(mut self, push: &'a [vk::PushConstantRange]) -> Self {
        self.push = Some(push);
        self
    }

    pub fn build(self) -> VulkanResult<PipelineLayout> {

        let push = self.push.expect("Missing Push");
        let layouts = self.layouts.unwrap_or(&[]);

        let create_info = vk::PipelineLayoutCreateInfo::default()
            .push_constant_ranges(push)
            .set_layouts(layouts);

        let layout = unsafe {
            profiling::scope!("vkCreatePipelineLayout");
            self.device
                .create_pipeline_layout(&create_info, None)
                .map_err(VulkanError::Unknown)
        }?;

        debug!(
            handle = ?layout,
            push = ?push,
            set_layout = ?layouts,
            "PipelineLayout created"
        );

        Ok(PipelineLayout { raw: layout })
    }
}
