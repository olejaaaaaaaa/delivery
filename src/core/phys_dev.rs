use ash::vk;

use crate::core::{Instance, VulkanError, VulkanResult};

pub struct PhysicalDevice {
    pub(crate) raw: vk::PhysicalDevice,
    pub(crate) props: vk::PhysicalDeviceProperties,
}

impl PhysicalDevice {
    pub fn new(instance: &Instance) -> VulkanResult<Self> {
        let phys_devs = unsafe {
            profiling::scope!("vkEnumeratePhysicalDevices");
            instance
                .raw
                .enumerate_physical_devices()
                .map_err(VulkanError::Unknown)
        }?;

        let gpu_priority: [vk::PhysicalDeviceType; 5] = [
            vk::PhysicalDeviceType::DISCRETE_GPU,
            vk::PhysicalDeviceType::INTEGRATED_GPU,
            vk::PhysicalDeviceType::VIRTUAL_GPU,
            vk::PhysicalDeviceType::CPU,
            vk::PhysicalDeviceType::OTHER,
        ];

        for ty in gpu_priority {
            for dev in &phys_devs {
                let props = unsafe {
                    profiling::scope!("vkGetPhysicalDeviceProperties");
                    instance.raw.get_physical_device_properties(*dev)
                };

                if props.device_type == ty {
                    return Ok(Self { raw: *dev, props });
                }
            }
        }

        Err(VulkanError::Unknown(vk::Result::from_raw(0)))
    }
}

impl std::ops::Deref for PhysicalDevice {
    type Target = ash::vk::PhysicalDevice;
    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}
