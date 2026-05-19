use std::sync::Arc;
use std::sync::Mutex;

use ash::vk;
use tracing::debug;

use crate::core::Surface;
use crate::core::VulkanError;

pub struct QueuePool {
    queues: Vec<Vec<Arc<Mutex<Queue>>>>,
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
                queue_family.push(Arc::new(Mutex::new(Queue {
                    raw: queue,
                    flags: prop.queue_flags,
                    is_present,
                    family_index: family_index as u32,
                    queue_index,
                })));
            }
            queues.push(queue_family);
        }

        QueuePool { queues }
    }

    pub fn get(&self, flags: vk::QueueFlags) -> Option<Arc<Mutex<Queue>>> {
        self.queues
            .iter()
            .flatten()
            .find(|q| {
                if let Ok(queue) = q.try_lock() {
                    return queue.flags.contains(flags);
                }
                false
            })
            .cloned()
    }

      pub fn get_wait(&self, flags: vk::QueueFlags) -> Option<Arc<Mutex<Queue>>> {
        self.queues
            .iter()
            .flatten()
            .find(|q| {
                if let Ok(queue) = q.lock() {
                    return queue.flags.contains(flags);
                }
                false
            })
            .cloned()
    }

    pub fn get_present(&self) -> Option<Arc<Mutex<Queue>>> {
        self.queues.iter().flatten().find(|q| {
            if let Ok(queue) = q.try_lock() {
                return queue.is_present && queue.flags.contains(vk::QueueFlags::GRAPHICS);
            }
            false
        }).cloned()
    }

    pub fn get_dedicated(&self, flags: vk::QueueFlags) -> Option<Arc<Mutex<Queue>>> {
        self.queues
            .iter()
            .flatten()
            .find(|q| {
                if let Ok(queue) = q.try_lock() {
                    return queue.flags == flags;
                }
                false
            })
            .cloned()
            .or_else(|| self.get(flags))
    }

    pub fn get_wait2(&self, flags: vk::QueueFlags) -> Option<Arc<Mutex<Queue>>> {
        loop {
            let mut locked_queues_count = 0;

            for q in self.queues.iter().flatten() {
                if let Ok(queue) = q.try_lock() {
                    if queue.flags.contains(flags) {
                        return Some(Arc::clone(q));
                    }
                } else {
                    locked_queues_count += 1;
                }
            }

            if locked_queues_count == 0 {
                return None;
            }

            std::thread::yield_now(); 
        }
    }

    pub fn graphics(&self) -> Option<Arc<Mutex<Queue>>> {
        self.get(vk::QueueFlags::GRAPHICS)
    }

    pub fn compute(&self) -> Option<Arc<Mutex<Queue>>> {
        self.get_dedicated(vk::QueueFlags::COMPUTE)
    }

    pub fn transfer(&self) -> Option<Arc<Mutex<Queue>>> {
        self.get_dedicated(vk::QueueFlags::TRANSFER)
    }
}
