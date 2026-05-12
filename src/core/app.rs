use std::ffi::CStr;

use ash::vk;
use tracing::{debug, info, warn};

use super::{VulkanError, VulkanResult};
use crate::core::errors::app::AppError;
use crate::core::ApiVersion;

const ENGINE_VERSION: u32 = 0;
const ENGINE_NAME: &CStr = c"Aluminium";
const APP_NAME: &CStr = c"App";
const APP_VERSION: u32 = 0;

pub struct App {
    pub(crate) create_info: vk::ApplicationInfo<'static>,
    pub(crate) entry: ash::Entry,
}

impl App {
    pub fn new() -> VulkanResult<App> {
        let entry = unsafe {
            ash::Entry::load().map_err(|e| VulkanError::App(AppError::LoadingVulkan(e)))
        }?;

        let available_api_version = unsafe {
            profiling::scope!("vkEnumerateInstanceVersion");
            entry
                .try_enumerate_instance_version()
                .map_err(|e| VulkanError::App(AppError::LoadingVulkanApiVersion(e)))?
                .unwrap_or(vk::API_VERSION_1_0)
        };

        debug!(
            "Max Vulkan Api version: {}",
            available_api_version.display_version()
        );

        // Downgrade from the highest available version
        // Using the latest version is quite dangerous
        let api_version = match available_api_version {
            version if version > vk::API_VERSION_1_3 => vk::API_VERSION_1_3,
            vk::API_VERSION_1_3 => vk::API_VERSION_1_2,
            vk::API_VERSION_1_2 => vk::API_VERSION_1_1,
            vk::API_VERSION_1_1 => vk::API_VERSION_1_0,
            _ => {
                warn!("GPU only supports the most minimal api version!");
                vk::API_VERSION_1_0
            },
        };

        let create_info = vk::ApplicationInfo::default()
            .api_version(api_version)
            .application_version(APP_VERSION)
            .application_name(APP_NAME)
            .engine_name(ENGINE_NAME)
            .engine_version(ENGINE_VERSION);

        info!(
            "Selected Vulkan Api version: {}",
            api_version.display_version()
        );

        Ok(App { create_info, entry })
    }
}
