use ash::vk;
use tracing::debug;

use super::{Device, VulkanError, VulkanResult};
use crate::core::debug;

pub struct Semaphore {
    pub raw: vk::Semaphore,
}

impl Semaphore {
    pub fn destroy(&self, device: &Device) {
        unsafe { device.destroy_semaphore(self.raw, None) };
        debug!(
            handle = ?self.raw,
            "Destroy Semaphore"
        );
    }
}

pub struct SemaphoreBuilder<'a> {
    device: &'a Device,
}

impl<'a> SemaphoreBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self { device }
    }

    pub fn build(self) -> VulkanResult<Semaphore> {
        let create_info = vk::SemaphoreCreateInfo::default();

        let semaphore = unsafe {
            profiling::scope!("vkCreateSemaphore");
            self.device
                .create_semaphore(&create_info, None)
                .map_err(VulkanError::Unknown)
        }?;

        Ok(Semaphore { raw: semaphore })
    }
}
