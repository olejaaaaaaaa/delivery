use ash::vk;
use tracing::debug;

use super::device::Device;
use super::{VulkanError, VulkanResult};

pub struct DescriptorPool {
    pub raw: vk::DescriptorPool,
}

impl DescriptorPool {
    pub fn destroy(&self, device: &Device) {
        unsafe { device.destroy_descriptor_pool(self.raw, None) };
        debug!(
            handle = ?self.raw,
            "Destroy DescriptorPool"
        );
    }

    pub fn create_descriptor_set(
        &self,
        device: &Device,
        layouts: &[vk::DescriptorSetLayout],
    ) -> VulkanResult<Vec<vk::DescriptorSet>> {
        #[cfg(debug_assertions)]
        {
            assert!(!layouts.is_empty(), "Cannot allocate 0 descriptor sets!");
        }

        let desc = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.raw)
            .set_layouts(layouts);

        unsafe {
            profiling::scope!("vkCreateDescriptorSet");
            device
                .allocate_descriptor_sets(&desc)
                .map_err(VulkanError::Unknown)
        }
    }
}

pub struct DescriptorPoolBuilder<'a> {
    device: &'a Device,
    flags: vk::DescriptorPoolCreateFlags,
    sizes: Option<&'a [vk::DescriptorPoolSize]>,
    max_sets: Option<u32>,
}

impl<'a> DescriptorPoolBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            device,
            flags: vk::DescriptorPoolCreateFlags::empty(),
            sizes: None,
            max_sets: None,
        }
    }

    pub fn flags(mut self, flags: vk::DescriptorPoolCreateFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn pool_sizes(mut self, sizes: &'a [vk::DescriptorPoolSize]) -> Self {
        self.sizes = Some(sizes);
        self
    }

    pub fn max_sets(mut self, sets: u32) -> Self {
        self.max_sets = Some(sets);
        self
    }

    pub fn build(self) -> VulkanResult<DescriptorPool> {
        let sizes = self.sizes.expect("Missing Pool sizes");
        let max_sets = self.max_sets.expect("Missing Max sets");
        let flags = self.flags;

        #[cfg(debug_assertions)]
        {
            if max_sets == 0 {
                panic!("DescriptorPool max sets cannot be 0");
            }

            if sizes.is_empty() {
                panic!("DescriptorPool must have at least one pool size");
            }
        }

        let create_info = vk::DescriptorPoolCreateInfo::default()
            .flags(flags)
            .pool_sizes(sizes)
            .max_sets(max_sets);

        let pool = unsafe {
            profiling::scope!("vkCreateDescriptorPool");
            self.device
                .create_descriptor_pool(&create_info, None)
                .map_err(VulkanError::Unknown)
        }?;

        debug!(
            handle = ?pool,
            sizes = ?sizes,
            max_sets = ?max_sets,
            flags = ?flags,
            "DescriptorPool created"
        );

        Ok(DescriptorPool { raw: pool })
    }
}
