use core::slice;
use std::borrow::Cow;
use std::ffi::{c_void, CStr, CString};
use std::thread;

use ash::vk;
use tracing::{error, warn};

pub struct DebugCallback {
    callback: vk::DebugUtilsMessengerEXT,
    loader: ash::ext::debug_utils::Instance,
}

impl DebugCallback {
    pub fn destroy(&self) {
        unsafe {
            self.loader
                .destroy_debug_utils_messenger(self.callback, None);
        };
    }

    pub fn new(entry: &ash::Entry, instance: &ash::Instance) -> Self {
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(debug_utils_messenger_callback));

        let loader = ash::ext::debug_utils::Instance::new(entry, instance);

        let callback = unsafe {
            loader
                .create_debug_utils_messenger(&debug_info, None)
                .unwrap()
        };

        DebugCallback { callback, loader }
    }
}

#[derive(Debug)]
/// From wgpu-hal, dual licensed Apache-2.0 OR MIT
///
/// The properties related to the validation layer needed for the
/// DebugUtilsMessenger for their workarounds
struct ValidationLayerProperties {
    /// Validation layer description, from `vk::LayerProperties`.
    layer_description: CString,

    /// Validation layer specification version, from `vk::LayerProperties`.
    layer_spec_version: u32,
}

/// From wgpu-hal, dual licensed Apache-2.0 OR MIT
///
/// User data needed by `instance::debug_utils_messenger_callback`.
///
/// When we create the [`vk::DebugUtilsMessengerEXT`], the `pUserData`
/// pointer refers to one of these values.
#[derive(Debug)]
pub struct DebugUtilsMessengerUserData {
    /// The properties related to the validation layer, if present
    validation_layer_properties: Option<ValidationLayerProperties>,

    /// If the OBS layer is present. OBS never increments the version of their
    /// layer, so there's no reason to have the version.
    has_obs_layer: bool,
}

unsafe extern "system" fn debug_utils_messenger_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    callback_data_ptr: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    user_data: *mut c_void,
) -> vk::Bool32 {
    if thread::panicking() {
        return vk::FALSE;
    }

    let cd = unsafe { &*callback_data_ptr };
    let user_data = unsafe { &*user_data.cast::<DebugUtilsMessengerUserData>() };

    const VUID_VKCMDENDDEBUGUTILSLABELEXT_COMMANDBUFFER_01912: i32 = 0x56146426;
    if cd.message_id_number == VUID_VKCMDENDDEBUGUTILSLABELEXT_COMMANDBUFFER_01912 {
        // https://github.com/KhronosGroup/Vulkan-ValidationLayers/issues/5671
        // Versions 1.3.240 through 1.3.250 return a spurious error here if
        // the debug range start and end appear in different command buffers.
        if let Some(layer_properties) = user_data.validation_layer_properties.as_ref() {
            if layer_properties.layer_description.as_ref() == c"Khronos Validation Layer"
                && layer_properties.layer_spec_version >= vk::make_api_version(0, 1, 3, 240)
                && layer_properties.layer_spec_version <= vk::make_api_version(0, 1, 3, 250)
            {
                return vk::FALSE;
            }
        }
    }

    // Silence Vulkan Validation error
    // "VUID-VkSwapchainCreateInfoKHR-pNext-07781" This happens when a
    // surface is configured with a size outside the allowed extent. It's a
    // false positive due to the inherent racy-ness of surface resizing.
    const VUID_VKSWAPCHAINCREATEINFOKHR_PNEXT_07781: i32 = 0x4c8929c1;
    if cd.message_id_number == VUID_VKSWAPCHAINCREATEINFOKHR_PNEXT_07781 {
        return vk::FALSE;
    }

    // Silence Vulkan Validation error
    // "VUID-VkRenderPassBeginInfo-framebuffer-04627" if the OBS layer is
    // enabled. This is a bug in the OBS layer. As the OBS layer does not have a
    // version number they increment, there is no way to qualify the suppression
    // of the error to a specific version of the OBS layer.
    //
    // See https://github.com/obsproject/obs-studio/issues/9353
    const VUID_VKRENDERPASSBEGININFO_FRAMEBUFFER_04627: i32 = 0x45125641;
    if cd.message_id_number == VUID_VKRENDERPASSBEGININFO_FRAMEBUFFER_04627
        && user_data.has_obs_layer
    {
        return vk::FALSE;
    }

    // Silence Vulkan Validation error
    // "VUID-vkCmdCopyImageToBuffer-pRegions-00184". While we aren't sure
    // yet, we suspect this is probably a VVL issue. https://github.com/KhronosGroup/Vulkan-ValidationLayers/issues/9276
    const VUID_VKCMDCOPYIMAGETOBUFFER_PREGIONS_00184: i32 = 0x45ef177c;
    if cd.message_id_number == VUID_VKCMDCOPYIMAGETOBUFFER_PREGIONS_00184 {
        return vk::FALSE;
    }

    // Silence Vulkan Validation error "VUID-StandaloneSpirv-None-10684".
    //
    // This is a bug. To prevent massive noise in the tests, lets suppress it
    // for now. https://github.com/gfx-rs/wgpu/issues/7696
    const VUID_STANDALONESPIRV_NONE_10684: i32 = 0xb210f7c2_u32 as i32;
    if cd.message_id_number == VUID_STANDALONESPIRV_NONE_10684 {
        return vk::FALSE;
    }

    let level = match message_severity {
        // We intentionally suppress info messages down to debug
        // so that users are not innundated with info messages from the runtime.
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => tracing::Level::TRACE,
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => tracing::Level::DEBUG,
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => tracing::Level::WARN,
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => tracing::Level::ERROR,
        _ => tracing::Level::WARN,
    };

    let message_id_name =
        unsafe { cd.message_id_name_as_c_str() }.map_or(Cow::Borrowed(""), CStr::to_string_lossy);
    let message = unsafe { cd.message_as_c_str() }.map_or(Cow::Borrowed(""), CStr::to_string_lossy);

    let _ = std::panic::catch_unwind(|| {
        // tracing::span!(level, &format!("{:?}", message));
    });

    if cd.queue_label_count != 0 {
        let labels =
            unsafe { slice::from_raw_parts(cd.p_queue_labels, cd.queue_label_count as usize) };
        let names = labels
            .iter()
            .flat_map(|dul_obj| unsafe { dul_obj.label_name_as_c_str() }.map(CStr::to_string_lossy))
            .collect::<Vec<_>>();

        let _ = std::panic::catch_unwind(|| {
            // tracing::span!(level, "\tqueues: {}", names.join(", "));
        });
    }

    if cd.cmd_buf_label_count != 0 {
        let labels =
            unsafe { slice::from_raw_parts(cd.p_cmd_buf_labels, cd.cmd_buf_label_count as usize) };
        let names = labels
            .iter()
            .flat_map(|dul_obj| unsafe { dul_obj.label_name_as_c_str() }.map(CStr::to_string_lossy))
            .collect::<Vec<_>>();

        let _ = std::panic::catch_unwind(|| {
            // error!(level, "\tcommand buffers: {}", names.join(", "));
        });
    }

    if cd.object_count != 0 {
        let labels = unsafe { slice::from_raw_parts(cd.p_objects, cd.object_count as usize) };
        // TODO: use color fields of `vk::DebugUtilsLabelExt`?
        let names = labels
            .iter()
            .map(|obj_info| {
                let name = unsafe { obj_info.object_name_as_c_str() }
                    .map_or(Cow::Borrowed("?"), CStr::to_string_lossy);

                format!(
                    "(type: {:?}, hndl: 0x{:x}, name: {})",
                    obj_info.object_type, obj_info.object_handle, name
                )
            })
            .collect::<Vec<_>>();

        let _ = std::panic::catch_unwind(|| {
            // error!(level, "\tobjects: {}", names.join(", "));
        });
    }

    #[cfg(feature = "validation_canary")]
    if cfg!(debug_assertions) && level == log::Level::Error {
        use alloc::string::ToString as _;

        // Set canary and continue
        crate::VALIDATION_CANARY.add(message.to_string());
    }

    vk::FALSE
}
