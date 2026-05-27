use crate::core::VulkanError;

pub enum LoadState<T> {
    Failed(VulkanError),
    Loading,
    Ready(T)
}