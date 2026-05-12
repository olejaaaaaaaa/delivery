use ash::vk;
use tracing::debug;

use super::device::Device;
use super::{VulkanError, VulkanResult};

pub struct CommandPool {
    pub raw: vk::CommandPool,
}

impl CommandPool {
    pub fn destroy(&self, device: &Device) {
        unsafe { device.destroy_command_pool(self.raw, None) };
        debug!(
            handle = ?self.raw,
            "Destroy CommandPool"
        );
    }

    pub fn allocate_cmd_buffers(
        &self,
        device: &Device,
        level: vk::CommandBufferLevel,
        count: u32,
    ) -> VulkanResult<Vec<vk::CommandBuffer>> {
        debug_assert!(count != 0, "Cannot create 0 command buffers!");

        let create_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.raw)
            .level(level)
            .command_buffer_count(count);

        let buffers = unsafe {
            profiling::scope!("vkAllocateCommandBuffers");
            device
                .allocate_command_buffers(&create_info)
                .map_err(VulkanError::Unknown)
        }?;

        Ok(buffers)
    }
}

pub struct CommandPoolBuilder<'a> {
    device: &'a Device,
    flags: vk::CommandPoolCreateFlags,
}

impl<'a> CommandPoolBuilder<'a> {
    pub fn reset(device: &'a Device) -> Self {
        Self {
            device,
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        }
    }

    pub fn build(self) -> VulkanResult<CommandPool> {
        let create_info = vk::CommandPoolCreateInfo::default().flags(self.flags);

        let pool = unsafe {
            profiling::scope!("vkCreateCommandPool");
            self.device
                .create_command_pool(&create_info, None)
                .map_err(VulkanError::Unknown)
        }?;

        debug!(
            handle = ?pool,
            flags = ?self.flags,
            "CommandPool created"
        );

        Ok(CommandPool { raw: pool })
    }
}
