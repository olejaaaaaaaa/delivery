use ash::vk;

use crate::core::{CommandPool, CommandPoolBuilder, Device, VulkanResult};

pub struct CommandPoolPerFrame {
    command_pool: CommandPool,
    command_buffers: Vec<Vec<vk::CommandBuffer>>,
}

impl CommandPoolPerFrame {
    pub fn destroy(&self, device: &Device) {
        self.command_pool.destroy(device);
    }

    pub fn new(device: &Device) -> VulkanResult<Self> {
        let pool = CommandPoolBuilder::reset(device).build()?;

        Ok(Self {
            command_pool: pool,
            command_buffers: vec![],
        })
    }

    pub fn allocate_cmd_buffers(
        &mut self,
        _device: &Device,
        image_index: u32,
        frame_count: usize,
        _pass_count: usize,
    ) -> VulkanResult<&Vec<vk::CommandBuffer>> {
        while self.command_buffers.len() < frame_count {
            self.command_buffers.push(vec![]);
        }

        // for i in &mut self.command_buffers {
        //     let dif = pass_count - i.len();
        //     if dif > 0 {
        //         let buffers = self
        //             .command_pool
        //             .create_cmd_buffers(device, dif as u32)?;
        //         i.extend(buffers);
        //     }
        // }

        Ok(&self.command_buffers[image_index as usize])
    }
}
