use std::collections::HashSet;
use std::ffi::CStr;

use ash::vk;
use tracing::{debug, info, warn};
use winit::raw_window_handle::HasDisplayHandle;

use super::app::App;
use super::debug::DebugCallback;
use super::{VulkanError, VulkanResult};
use crate::core::InstanceError;

/// Unsafe Wrapper around [`vk::Instance`]
/// Required manually destroy before Drop
pub struct Instance {
    pub(crate) raw: ash::Instance,
    pub(crate) extensions: HashSet<&'static CStr>,
    pub(crate) layers: HashSet<&'static CStr>,
    pub(crate) debug_callback: Option<DebugCallback>,
}

impl Instance {
    pub fn destroy(&self) {
        if let Some(debug) = &self.debug_callback {
            debug.destroy();
        }
        unsafe { self.raw.destroy_instance(None) };
        debug!("Instance destroyed");
    }
}

impl Instance {
    pub fn check_supported_layers(&self, layers: &[&'static CStr]) -> bool {
        for i in layers {
            if !self.layers.contains(i) {
                return false;
            }
        }
        true
    }

    pub fn check_supported_extensions(&self, extensions: &[&'static CStr]) -> bool {
        for i in extensions {
            if !self.extensions.contains(i) {
                return false;
            }
        }
        true
    }

    fn get_instance_extensions(
        window: &winit::window::Window,
        app: &App,
    ) -> VulkanResult<HashSet<&'static CStr>> {
        let mut extensions = HashSet::new();

        let available_extensions = unsafe {
            app.entry
                .enumerate_instance_extension_properties(None)
                .map_err(VulkanError::Unknown)
        }?;

        let mut available_extension_names = HashSet::new();

        for i in &available_extensions {
            if let Ok(name) = i.extension_name_as_c_str() {
                available_extension_names.insert(name);
            }
        }

        debug!(
            "Available Instance Extensions: {:#?}",
            available_extension_names
        );

        let required_extensions = [
            c"VK_KHR_get_physical_device_properties2",
            #[cfg(all(feature = "validation_layer", not(target_os = "android")))]
            c"VK_EXT_debug_utils",
            #[cfg(all(feature = "validation_layer", target_os = "android"))]
            c"VK_EXT_debug_report",
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            c"VK_KHR_portability_enumeration",
        ];

        for i in required_extensions {
            if !available_extension_names.contains(i) {
                return Err(VulkanError::Instance(
                    InstanceError::MissingRequiredExtension(i.to_str().unwrap().to_string()),
                ));
            } else {
                extensions.insert(i);
            }
        }

        let window_extensions = ash_window::enumerate_required_extensions(
            window
                .display_handle()
                .expect("Error get raw window display handle")
                .into(),
        )
        .map_err(VulkanError::Unknown)?
        .iter()
        .map(|ptr| unsafe { CStr::from_ptr(*ptr) })
        .collect::<Vec<_>>();

        for i in &window_extensions {
            if !available_extension_names.contains(i) {
                return Err(VulkanError::Instance(
                    InstanceError::MissingRequiredExtension(i.to_str().unwrap().to_string()),
                ));
            }
        }

        extensions.extend(window_extensions);

        Ok(extensions)
    }

    fn get_instance_layers(app: &App) -> VulkanResult<HashSet<&'static CStr>> {
        let mut layers = HashSet::new();
        let available_layers = unsafe {
            app.entry
                .enumerate_instance_layer_properties()
                .map_err(VulkanError::Unknown)
        }?;

        let mut available_layer_names = HashSet::new();

        for i in &available_layers {
            if let Ok(name) = i.layer_name_as_c_str() {
                available_layer_names.insert(name);
            }
        }

        debug!("Available Instance layers: {:#?}", available_layer_names);

        let optional_layers = [
            #[cfg(any(feature = "layer_khronos_validation", debug_assertions))]
            c"VK_LAYER_KHRONOS_validation",
        ];

        for i in optional_layers {
            if !available_layer_names.contains(i) {
                warn!("Instance layer {} is not available", i.to_str().unwrap());
            } else {
                layers.insert(i);
            }
        }

        Ok(layers)
    }

    pub fn new(window: &winit::window::Window, app: &App) -> VulkanResult<Instance> {
        let layers = Self::get_instance_layers(app)?;
        let p_layers = layers
            .iter()
            .map(|name| (*name).as_ptr())
            .collect::<Vec<_>>();

        let extensions = Self::get_instance_extensions(window, app)?;
        let p_extensions = extensions
            .iter()
            .map(|name| (*name).as_ptr())
            .collect::<Vec<_>>();

        let flags = cfg!(any(target_os = "macos", target_os = "ios"))
            .then_some(vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR)
            .unwrap_or_default();

        let create_info = vk::InstanceCreateInfo::default()
            .enabled_layer_names(&p_layers)
            .enabled_extension_names(&p_extensions)
            .application_info(&app.create_info)
            .flags(flags);

        let instance = unsafe {
            app.entry
                .create_instance(&create_info, None)
                .map_err(VulkanError::Unknown)
        }?;

        // Warn! Only 33% Android devices supported VK_EXT_debug_utils
        // TODO: For android use VK_EXT_debug_report
        let debug_callback = if extensions.contains(&c"VK_EXT_debug_utils")
            && layers.contains(&c"VK_LAYER_KHRONOS_validation")
        {
            Some(DebugCallback::new(&app.entry, &instance))
        } else {
            None
        };

        info!(
            layers = ?layers,
            extensions = ?extensions,
            flags = ?flags,
            "Instance created"
        );

        Ok(Instance {
            raw: instance,
            layers,
            extensions,
            debug_callback,
        })
    }
}
