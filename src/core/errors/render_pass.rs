use ash::vk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderPassError {
    #[error("Failed create RenderPass (Vulkan error: {0:?})")]
    CreateRenderPass(vk::Result),
}
