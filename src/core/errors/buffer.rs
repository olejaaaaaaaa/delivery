use ash::vk::Result;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BufferError {
    #[error("Error update GpuBuffer")]
    Update(ash::vk::Result),
    #[error("Error get support vulkan api version")]
    LoadingVulkanApiVersion(Result),
}
