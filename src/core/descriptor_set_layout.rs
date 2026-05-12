use ash::vk;

use super::device::Device;
use super::{VulkanError, VulkanResult};

pub struct DescriptorSetLayout {
    pub raw: vk::DescriptorSetLayout,
}

pub struct DescriptorSetLayoutBuilder<'a> {
    device: &'a Device,
    flags: vk::DescriptorSetLayoutCreateFlags,
    push_next: Option<&'a mut vk::DescriptorSetLayoutBindingFlagsCreateInfo<'a>>,
    bindings: Vec<vk::DescriptorSetLayoutBinding<'static>>,
}

impl<'a> DescriptorSetLayoutBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            flags: vk::DescriptorSetLayoutCreateFlags::empty(),
            push_next: None,
            bindings: vec![],
            device,
        }
    }

    pub fn bindings(mut self, bindings: Vec<vk::DescriptorSetLayoutBinding<'static>>) -> Self {
        self.bindings = bindings;
        self
    }

    pub fn push_next(
        mut self,
        value: &'a mut vk::DescriptorSetLayoutBindingFlagsCreateInfo<'a>,
    ) -> Self {
        self.push_next = Some(value);
        self
    }

    pub fn flags(mut self, flags: vk::DescriptorSetLayoutCreateFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn build(self) -> VulkanResult<DescriptorSetLayout> {
        debug_assert!(!self.bindings.is_empty(), "Bindings empty!");

        let mut create_info = vk::DescriptorSetLayoutCreateInfo::default()
            .flags(self.flags)
            .bindings(&self.bindings);

        if let Some(next) = self.push_next {
            create_info = create_info.push_next(next);
        }

        let layout = unsafe {
            profiling::scope!("vkCreateDescriptorSetLayout");
            self.device
                .create_descriptor_set_layout(&create_info, None)
                .map_err(VulkanError::Unknown)
        }?;

        Ok(DescriptorSetLayout { raw: layout })
    }
}
