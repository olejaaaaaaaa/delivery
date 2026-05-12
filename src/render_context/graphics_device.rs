use crate::core::{App, Device, Instance, PhysicalDevice, QueuePool};

/// Wraps Vulkan device and instance with application-specific data
/// Provides convenient access to queues and device properties
pub struct GraphicsDevice {
    #[allow(dead_code)]
    /// Application info (name, version, etc.)
    pub(crate) app: App,
    /// Pool all of queues (graphics, compute, transfer)
    pub(crate) queue_pool: QueuePool,
    /// Vulkan instance
    pub(crate) instance: Instance,
    // Physical device
    pub(crate) phys_dev: PhysicalDevice,
    /// Logical device
    pub(crate) logical_device: Device,
    // The number of frames that can still be prepared while
    // the GPU is rendering the previous frame
    pub(crate) frame_in_flight: usize,
}

impl std::ops::Deref for GraphicsDevice {
    type Target = Device;
    fn deref(&self) -> &Self::Target {
        &self.logical_device
    }
}
