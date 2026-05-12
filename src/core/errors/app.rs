use ash::vk::Result;
use ash::LoadingError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Error dynamic load vulkan entry point")]
    LoadingVulkan(LoadingError),
    #[error("Error get support vulkan api version")]
    LoadingVulkanApiVersion(Result),
    #[error("Missing App Name")]
    MissingAppName,
    #[error("Missing Engine Name")]
    MissingEngineName,
    #[error("Missing Engine Version")]
    MissingEngineVersion,
    #[error("Missing App Version")]
    MissingAppVersion,
    #[error("Minimal vulkan api version is not supported")]
    Api(u32),
}
