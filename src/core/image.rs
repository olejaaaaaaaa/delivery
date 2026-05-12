use ash::vk;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme};
use gpu_allocator::MemoryLocation;
use tracing::{debug, warn};

use super::device::Device;
use super::{VulkanError, VulkanResult};

pub struct Image {
    pub(crate) raw: vk::Image,
    pub(crate) allocation: Option<Allocation>,
}

impl Image {
    pub fn destroy(&mut self, device: &Device) {
        if let Some(allocation) = self.allocation.take() {
            debug!(
                handle = ?self.raw,
                bytes = allocation.size(),
                "Image destroyed"
            );
            let _ = {
                profiling::scope!("vkFreeImageMemory");
                device.allocator.lock().free(allocation)
            };
            unsafe {
                profiling::scope!("vkDestroyImage");
                device.destroy_image(self.raw, None);
            }
        } else {
            warn!("Double free detected!")
        }
    }
}

pub struct ImageBuilder<'a> {
    pub device: &'a Device,
    pub layout: Option<vk::ImageLayout>,
    pub samples: Option<vk::SampleCountFlags>,
    pub array_layers: Option<u32>,
    pub extent: Option<vk::Extent3D>,
    pub format: Option<vk::Format>,
    pub usage: Option<vk::ImageUsageFlags>,
    pub image_type: Option<vk::ImageType>,
}

impl<'a> ImageBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            device,
            layout: None,
            samples: None,
            array_layers: None,
            extent: None,
            format: None,
            usage: None,
            image_type: None,
        }
    }

    pub fn init_layout(mut self, layout: vk::ImageLayout) -> Self {
        self.layout = Some(layout);
        self
    }

    pub fn usage(mut self, usage: vk::ImageUsageFlags) -> Self {
        self.usage = Some(usage);
        self
    }

    pub fn extent(mut self, extent: vk::Extent3D) -> Self {
        self.extent = Some(extent);
        self
    }

    pub fn array_layers(mut self, layers: u32) -> Self {
        self.array_layers = Some(layers);
        self
    }

    pub fn format(mut self, format: vk::Format) -> Self {
        self.format = Some(format);
        self
    }

    pub fn image_type(mut self, ty: vk::ImageType) -> Self {
        self.image_type = Some(ty);
        self
    }

    pub fn build(self) -> VulkanResult<Image> {
        let usage = self.usage.expect("Missing Usage");
        let extent = self.extent.expect("Missing Extent");
        let format = self.format.expect("Missing Format");
        let image_type = self.image_type.unwrap_or(vk::ImageType::TYPE_2D);
        let array_layers = self.array_layers.unwrap_or(1);
        let samples = self.samples.unwrap_or(vk::SampleCountFlags::TYPE_1);
        let layout = self.layout.unwrap_or(vk::ImageLayout::UNDEFINED);

        let create_info = vk::ImageCreateInfo::default()
            .mip_levels(1)
            .samples(samples)
            .array_layers(array_layers)
            .extent(extent)
            .format(format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(layout)
            .image_type(image_type)
            .usage(usage);

        let image = unsafe {
            profiling::scope!("vkCreateImage");
            self.device
                .raw
                .create_image(&create_info, None)
                .map_err(VulkanError::Unknown)?
        };

        let requirements = unsafe { self.device.raw.get_image_memory_requirements(image) };

        let mut allocator = self.device.allocator.lock();

        let allocation = {
            profiling::scope!("vkAllocateImage");
            allocator
                .allocate(&AllocationCreateDesc {
                    name: "Image",
                    requirements,
                    location: MemoryLocation::GpuOnly,
                    linear: false,
                    allocation_scheme: AllocationScheme::GpuAllocatorManaged,
                })
                .map_err(|_| {
                    unsafe {
                        self.device.destroy_image(image, None);
                    }
                    VulkanError::Unknown(vk::Result::from_raw(0))
                })?
        };

        unsafe {
            profiling::scope!("vkBindImageMemory");
            self.device
                .raw
                .bind_image_memory(image, allocation.memory(), allocation.offset())
                .map_err(|e| {
                    self.device.raw.destroy_image(image, None);
                    VulkanError::Unknown(e)
                })?;
        }

        debug!(
            handle = ?image,
            bytes = allocation.size(),
            extent = ?extent,
            format = ?format,
            image_type = ?image_type,
            usage = ?usage,
            "Image created"
        );

        Ok(Image {
            raw: image,
            allocation: Some(allocation),
        })
    }
}
