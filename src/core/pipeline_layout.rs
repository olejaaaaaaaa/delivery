use ash::vk;
use tracing::debug;

use super::{Device, VulkanError, VulkanResult};

pub struct PipelineLayout {
    pub(crate) raw: vk::PipelineLayout,
}

pub struct PipelineLayoutBuilder<'a> {
    layout: Vec<vk::DescriptorSetLayout>,
    push: Vec<vk::PushConstantRange>,
    device: &'a Device,
}

impl<'a> PipelineLayoutBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            push: vec![],
            layout: vec![],
            device,
        }
    }

    pub fn set_layouts(mut self, layouts: Vec<vk::DescriptorSetLayout>) -> Self {
        self.layout = layouts;
        self
    }

    pub fn push_constant(mut self, push: Vec<vk::PushConstantRange>) -> Self {
        self.push = push;
        self
    }

    pub fn build(self) -> VulkanResult<PipelineLayout> {
        let create_info = vk::PipelineLayoutCreateInfo::default()
            .push_constant_ranges(&self.push)
            .set_layouts(&self.layout);

        let layout = unsafe {
            profiling::scope!("vkCreatePipelineLayout");
            self.device
                .create_pipeline_layout(&create_info, None)
                .map_err(VulkanError::Unknown)
        }?;

        debug!(
            handle = ?layout,
            push = ?self.push,
            set_layout = ?self.layout,
            "PipelineLayout created"
        );

        Ok(PipelineLayout { raw: layout })
    }
}
