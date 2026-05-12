use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShaderError {
    #[error("EERRROR AAAA")]
    UnsupportedType,
    #[error("Error create reflection module")]
    ShaderReflectionModuleCreationFailed(String),
    #[error("Error reflection shader")]
    ShaderReflection,
    #[error("Not valid shader extension")]
    ShaderInvalidExtension,
    #[error("Not valid path to shader")]
    ShaderInvalidPath,
    #[error("Error read shader to end")]
    ShaderReadToEnd,
    #[error("Error create Shader with not valid unicode like name")]
    ShaderNameNotValidUnicode,
    #[error("Error create Shader")]
    ShaderCreationFailed(ash::vk::Result),
}
