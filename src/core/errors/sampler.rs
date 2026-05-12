use thiserror::Error;

#[derive(Debug, Error)]
pub enum SamplerError {
    #[error("Error create Sampler")]
    SamplerCreationFailed(ash::vk::Result),
}
