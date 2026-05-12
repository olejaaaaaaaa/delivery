use ash::vk;
use tracing::debug;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use super::app::App;
use super::instance::Instance;
use super::{VulkanError, VulkanResult};

pub struct Surface {
    pub raw: vk::SurfaceKHR,
    pub loader: ash::khr::surface::Instance,
}

impl Surface {
    pub fn new(
        app: &App,
        instance: &Instance,
        window: &winit::window::Window,
    ) -> VulkanResult<Surface> {
        let surface = unsafe {
            profiling::scope!("vkCreateNativeSurface");
            ash_window::create_surface(
                &app.entry,
                &instance.raw,
                window.display_handle().unwrap().into(),
                window.window_handle().unwrap().into(),
                None,
            )
            .map_err(VulkanError::Unknown)?
        };

        let loader = ash::khr::surface::Instance::new(&app.entry, &instance.raw);

        Ok(Surface {
            raw: surface,
            loader,
        })
    }
}

impl Surface {
    pub fn destroy(&self) {
        unsafe { self.loader.destroy_surface(self.raw, None) };
        debug!(
            handle = ?self.raw,
            "Surface destroyed"
        );
    }

    pub fn get_physical_device_surface_capabilities(
        &self,
        phys_dev: vk::PhysicalDevice,
    ) -> VulkanResult<vk::SurfaceCapabilitiesKHR> {
        unsafe {
            self.loader
                .get_physical_device_surface_capabilities(phys_dev, self.raw)
                .map_err(VulkanError::Unknown)
        }
    }

    pub fn get_physical_device_surface_formats(
        &self,
        phys_dev: vk::PhysicalDevice,
    ) -> VulkanResult<Vec<vk::SurfaceFormatKHR>> {
        unsafe {
            self.loader
                .get_physical_device_surface_formats(phys_dev, self.raw)
                .map_err(VulkanError::Unknown)
        }
    }
}
