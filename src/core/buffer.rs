use ash::vk::{self, Handle};
use bytemuck::{Pod, Zeroable};
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme};
use gpu_allocator::MemoryLocation;
use tracing::{debug, warn};

use super::{Device, VulkanError, VulkanResult};

pub struct GpuBuffer {
    pub raw: vk::Buffer,
    pub allocation: Option<Allocation>,
}

impl GpuBuffer {
    /// Copy slice as raw bytes into [`vk::Buffer`]
    pub fn upload_data<T: Pod + Zeroable>(&mut self, data: &[T]) -> VulkanResult<()> {
        let size = std::mem::size_of_val(data);

        assert!(size != 0, "Cannot create empty buffer");

        let dst = self
            .allocation
            .as_ref()
            .expect("Buffer alredy free")
            .mapped_ptr()
            .expect("Buffer is not host-visible or not mapped")
            .cast::<u8>()
            .as_ptr();

        unsafe {
            profiling::scope!("Upload bytes");
            std::ptr::copy_nonoverlapping(data.as_ptr().cast::<u8>(), dst, size);
        }

        Ok(())
    }

    pub fn destroy(&mut self, device: &Device) {
        if let Some(allocation) = self.allocation.take() {
            let size = allocation.size();
            let _ = device.allocator.lock().free(allocation);
            unsafe {
                device.destroy_buffer(self.raw, None);
            }
            debug!(
                handle = ?self.raw,
                bytes = size,
                "Buffer destroyed"
            );
        } else {
            warn!("Double free detected!");
        }
    }
}

pub struct GpuBufferBuilder<'a> {
    size: Option<u64>,
    usage: Option<vk::BufferUsageFlags>,
    location: MemoryLocation,
    device: &'a Device,
}

impl<'a> GpuBufferBuilder<'a> {
    pub fn cpu_only(device: &'a Device) -> Self {
        GpuBufferBuilder {
            device,
            usage: None,
            size: None,
            location: MemoryLocation::CpuToGpu,
        }
    }

    pub fn gpu_to_cpu(device: &'a Device) -> Self {
        GpuBufferBuilder {
            size: None,
            usage: None,
            location: MemoryLocation::GpuToCpu,
            device,
        }
    }

    pub fn gpu_only(device: &'a Device) -> Self {
        GpuBufferBuilder {
            device,
            usage: None,
            size: None,
            location: MemoryLocation::GpuOnly,
        }
    }

    pub fn usage(mut self, usage: vk::BufferUsageFlags) -> Self {
        self.usage = Some(usage);
        self
    }

    pub fn size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    pub fn build(&self) -> VulkanResult<GpuBuffer> {
        let size = self.size.expect("Missing size");
        let usage = self.usage.expect("Missing usage");

        debug_assert_ne!(size, 0, "Buffer size cannot be zero");
        debug_assert!(!usage.is_empty(), "Buffer usage cannot be empty");

        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            self.device
                .raw
                .create_buffer(&buffer_info, None)
                .map_err(VulkanError::Unknown)?
        };

        let requirements = unsafe { self.device.raw.get_buffer_memory_requirements(buffer) };

        let allocation = {
            let allocator = &mut self.device.allocator.lock();
            allocator
                .allocate(&AllocationCreateDesc {
                    name: "GpuBuffer",
                    requirements,
                    location: self.location,
                    linear: true,
                    allocation_scheme: AllocationScheme::GpuAllocatorManaged,
                })
                .unwrap()
        };

        unsafe {
            profiling::scope!("vkBindBufferMemory");
            self.device
                .raw
                .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
                .map_err(VulkanError::Unknown)?;
        }

        debug!(
            handle = ?buffer,
            size = ?size,
            usage = ?usage,
            location = ?self.location,
            "Buffer created"
        );

        Ok(GpuBuffer {
            raw: buffer,
            allocation: Some(allocation),
        })
    }
}
