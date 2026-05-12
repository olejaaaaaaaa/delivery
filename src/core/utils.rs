pub trait ApiVersion {
    fn display_version(&self) -> String;
}

impl ApiVersion for u32 {
    fn display_version(&self) -> String {
        let variant = ash::vk::api_version_variant(*self);
        let major = ash::vk::api_version_major(*self);
        let minor = ash::vk::api_version_minor(*self);
        let patch = ash::vk::api_version_patch(*self);
        format!("{}.{}.{}.{}", variant, major, minor, patch)
    }
}
