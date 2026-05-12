use ash::vk;
use tracing::debug;

use super::{Device, VulkanError, VulkanResult};

pub struct Sampler {
    #[allow(dead_code)]
    pub raw: vk::Sampler,
}

impl Sampler {
    #[allow(dead_code)]
    pub fn destroy(&self, device: &Device) {
        unsafe { device.destroy_sampler(self.raw, None) };
    }
}

#[allow(dead_code)]
pub struct SamplerBuilder<'a> {
    device: &'a Device,
    anisotropy_enable: bool,
    address_mode_u: vk::SamplerAddressMode,
    address_mode_v: vk::SamplerAddressMode,
    address_mode_w: vk::SamplerAddressMode,
    border_color: vk::BorderColor,
}

impl<'a> SamplerBuilder<'a> {
    pub fn repeat(device: &'a Device) -> Self {
        Self {
            device,
            anisotropy_enable: false,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            border_color: vk::BorderColor::FLOAT_OPAQUE_WHITE,
        }
    }

    pub fn build(self) -> VulkanResult<Sampler> {
        let create_info = vk::SamplerCreateInfo::default()
            .address_mode_u(self.address_mode_u)
            .address_mode_v(self.address_mode_v)
            .address_mode_w(self.address_mode_w)
            .anisotropy_enable(self.anisotropy_enable)
            .border_color(self.border_color);

        let sampler = unsafe {
            profiling::scope!("vkCreateSampler");
            self.device
                .create_sampler(&create_info, None)
                .map_err(VulkanError::Unknown)?
        };

        debug!(
            handle = ?sampler,
            address_mode_u = ?self.address_mode_u,
            address_mode_v = ?self.address_mode_v,
            address_mode_w = ?self.address_mode_w,
            anisotropy_enable = self.anisotropy_enable,
            border_color = ?self.border_color,
            "Sampler created"
        );

        Ok(Sampler { raw: sampler })
    }
}
