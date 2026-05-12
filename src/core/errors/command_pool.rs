use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandPoolError {
    #[error("Error create CommandPool")]
    CommandPoolCreationFailed(ash::vk::Result),
    #[error("Error create CommandBuffers")]
    CommandBuffersCreationFailed(ash::vk::Result),
}
