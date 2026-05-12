use super::device::Device;
use super::{Fence, FenceBuilder, Semaphore, SemaphoreBuilder, VulkanResult};

/// Synchronization primitives for a single frame in flight
/// Contains semaphores and fence needed to coordinate GPU work
pub struct FrameSync {
    /// Signaled when swapchain image is ready for rendering
    pub image_available: Semaphore,
    /// Signaled when rendering commands have finished execution
    pub render_finished: Semaphore,
    /// Ensures CPU waits for GPU to finish processing this frame
    pub in_flight_fence: Fence,
}

impl FrameSync {
    /// Creates a new set of synchronization objects for one frame
    pub fn new(device: &Device) -> VulkanResult<FrameSync> {
        Ok(FrameSync {
            image_available: SemaphoreBuilder::new(device).build()?,
            render_finished: SemaphoreBuilder::new(device).build()?,
            // Fence starts signaled so first frame doesn't wait
            in_flight_fence: FenceBuilder::signaled(device).build()?,
        })
    }

    /// Destroy `FrameSync`
    pub fn destroy(&self, device: &Device) {
        self.image_available.destroy(device);
        self.render_finished.destroy(device);
        self.in_flight_fence.destroy(device);
    }
}
