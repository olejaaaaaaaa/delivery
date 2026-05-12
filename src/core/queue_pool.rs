use ash::vk;
use tracing::debug;

use crate::core::Surface;
use crate::core::VulkanError;

pub struct QueuePool {
    queues: Vec<Vec<Queue>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Queue {
    pub raw: vk::Queue,
    pub flags: vk::QueueFlags,
    pub is_present: bool,
    pub family_index: u32,
    pub queue_index: u32,
}

impl QueuePool {
    pub fn new(
        device: &ash::Device,
        phys_dev: &vk::PhysicalDevice,
        surface: &Surface,
        props: &[vk::QueueFamilyProperties],
    ) -> Self {
        let mut queues = vec![];

        for (family_index, prop) in props.iter().enumerate() {
            let mut queue_family = vec![];
            for queue_index in 0..prop.queue_count {
                let queue = unsafe { device.get_device_queue(family_index as u32, queue_index) };
                let is_present = unsafe {
                    surface
                        .loader
                        .get_physical_device_surface_support(
                            *phys_dev,
                            family_index as u32,
                            surface.raw,
                        )
                        .map_err(VulkanError::Unknown)
                        .unwrap()
                };
                queue_family.push(Queue {
                    raw: queue,
                    flags: prop.queue_flags,
                    is_present,
                    family_index: family_index as u32,
                    queue_index,
                });
            }
            queues.push(queue_family);
        }

        debug!("Queues: {:#?}", queues);

        QueuePool { queues }
    }

    pub fn get(&self, flags: vk::QueueFlags) -> Option<Queue> {
        self.queues
            .iter()
            .flatten()
            .find(|q| q.flags.contains(flags))
            .cloned()
    }

    pub fn get_present(&self) -> Option<Queue> {
        self.queues.iter().flatten().find(|q| q.is_present && q.flags.contains(vk::QueueFlags::GRAPHICS)).cloned()
    }

    pub fn get_dedicated(&self, flags: vk::QueueFlags) -> Option<Queue> {
        self.queues
            .iter()
            .flatten()
            .find(|q| q.flags == flags)
            .copied()
            .or_else(|| self.get(flags))
    }

    pub fn graphics(&self) -> Option<Queue> {
        self.get(vk::QueueFlags::GRAPHICS)
    }

    pub fn compute(&self) -> Option<Queue> {
        self.get_dedicated(vk::QueueFlags::COMPUTE)
    }

    pub fn transfer(&self) -> Option<Queue> {
        self.get_dedicated(vk::QueueFlags::TRANSFER)
    }
}
