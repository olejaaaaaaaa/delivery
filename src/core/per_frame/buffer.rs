use ash::vk;

use crate::core::{Device, GpuBuffer, GpuBufferBuilder, VulkanResult};

pub struct PerFrameBuffer {
    pub(crate) buffers: Vec<GpuBuffer>,
}

pub struct PerFrameBufferBuilder<'a> {
    device: &'a Device,
    frame_count: Option<usize>,
    buffer_size: Option<u64>,
    usage: Option<vk::BufferUsageFlags>,
}

impl<'a> PerFrameBufferBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            device,
            frame_count: None,
            buffer_size: None,
            usage: None,
        }
    }

    pub fn frame_count(mut self, frame_count: usize) -> Self {
        self.frame_count = Some(frame_count);
        self
    }

    pub fn buffer_size(mut self, size: u64) -> Self {
        self.buffer_size = Some(size);
        self
    }

    pub fn usage(mut self, usage: vk::BufferUsageFlags) -> Self {
        self.usage = Some(usage);
        self
    }

    pub fn build(self) -> VulkanResult<PerFrameBuffer> {
        let frame_count = self.frame_count.expect("Missing Frame count");
        let usage = self.usage.expect("Missing Usage");
        let size = self.buffer_size.expect("Missing Buffer size");

        let mut buffers = Vec::with_capacity(frame_count);

        for _ in 0..frame_count {
            buffers.push(
                GpuBufferBuilder::cpu_only(self.device)
                    .size(size)
                    .usage(usage)
                    .build()?,
            );
        }

        Ok(PerFrameBuffer { buffers })
    }
}

impl PerFrameBuffer {
    pub fn get_mut(&mut self, image_index: u32) -> &mut GpuBuffer {
        let len = self.buffers.len();
        &mut self.buffers[image_index as usize % len]
    }

    pub fn destroy(&mut self, device: &Device) {
        for mut i in self.buffers.drain(..) {
            i.destroy(device);
        }
    }
}
