use ash::vk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LogicalDeviceError {
    #[error("Failed create logical devices (Vulkan error: {0:?})")]
    CreateDevice(vk::Result),
    #[error("Required Vulkan extension not available: {0}")]
    MissingRequiredExtension(String),
}
