use std::error::Error;
use std::io::Read;
use std::path::Path;

use ash::vk;
use tracing::debug;

use super::device::Device;
use super::{VulkanError, VulkanResult};

pub struct ShaderModule {
    pub(crate) raw: vk::ShaderModule,
}

impl ShaderModule {
    pub fn destroy_shader(&self, device: &Device) {
        unsafe {
            device.destroy_shader_module(self.raw, None);
        }
        debug!(
            handle = ?self.raw,
            "Shader destroyed"
        )
    }
}

pub struct ShaderBuilder<'a> {
    device: &'a Device,
    bytecode: Option<&'a [u32]>,
}

impl<'a> ShaderBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            device,
            bytecode: None,
        }
    }

    pub fn bytecode(mut self, bytecode: &'a [u32]) -> Self {
        self.bytecode = Some(bytecode);
        self
    }

    pub fn build(self) -> VulkanResult<ShaderModule> {
        let device = self.device;
        let code = self.bytecode.expect("Missing SPIR-V bytecode");

        let create_info = vk::ShaderModuleCreateInfo::default().code(code);

        let shader = unsafe {
            profiling::scope!("vkCreateShaderModule");
            device
                .create_shader_module(&create_info, None)
                .map_err(VulkanError::Unknown)?
        };

        Ok(ShaderModule { raw: shader })
    }
}

pub(crate) fn read_shader_from_bytes(bytes: &[u8]) -> Result<Vec<u32>, Box<dyn Error>> {
    let mut cursor = std::io::Cursor::new(bytes);
    Ok(ash::util::read_spv(&mut cursor)?)
}

pub(crate) fn load_spv<T: AsRef<Path>>(path: T) -> Result<Vec<u32>, Box<dyn Error>> {
    let mut file = std::fs::File::open(path)?;
    let mut text = Vec::new();
    file.read_to_end(&mut text)?;

    assert_eq!(text.len() % 4, 0);
    assert_eq!(
        0x07230203,
        u32::from_le_bytes([text[0], text[1], text[2], text[3]])
    );

    Ok(read_shader_from_bytes(&text)?)
}
