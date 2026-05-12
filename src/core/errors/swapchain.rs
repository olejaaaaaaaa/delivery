use thiserror::Error;

#[derive(Debug, Error)]
pub enum SwapchainError {
    #[error("Error create Swapchain")]
    SwapchainCreationFailed(ash::vk::Result),
    #[error("Error out of date")]
    SwapchainOutOfDateKhr,
    #[error("Suboptimal")]
    SwapchainSubOptimal,
}
