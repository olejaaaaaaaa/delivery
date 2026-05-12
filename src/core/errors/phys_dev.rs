use ash::vk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PhysicalDeviceError {
    #[error("Failed to enumerate physical devices (Vulkan error: {0:?})")]
    EnumeratePhysicalDeviceFailed(vk::Result),
    #[error("Failed to get device extension properties (Vulkan error: {0:?})")]
    EnumerateDeviceExtensionPropertiesFailed(vk::Result),
    #[error("Failed to get device layer properties (Vulkan error: {0:?})")]
    EnumerateDeviceLayerPropertiesFailed(vk::Result),
}
