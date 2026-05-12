use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use ash::vk;

use super::Device;

#[allow(dead_code)]
pub struct PipelineCache {
    pub raw: vk::PipelineCache,
}

impl PipelineCache {
    #[allow(dead_code)]
    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_pipeline_cache(self.raw, None);
        }
    }

    #[allow(dead_code)]
    pub fn new(device: &Device) -> Result<Self, vk::Result> {
        let cache_info = vk::PipelineCacheCreateInfo::default()
            .flags(vk::PipelineCacheCreateFlags::empty())
            .initial_data(&[]);

        let cache = unsafe { device.create_pipeline_cache(&cache_info, None)? };

        Ok(Self { raw: cache })
    }

    #[allow(dead_code)]
    pub fn from_file(device: &Device, path: &Path) -> Result<Self, vk::Result> {
        let initial_data = if path.exists() {
            let mut file = File::open(path).unwrap();
            let mut data = Vec::new();
            file.read_to_end(&mut data).unwrap();
            data
        } else {
            return Err(vk::Result::from_raw(0));
        };

        let cache_info = vk::PipelineCacheCreateInfo::default()
            .flags(vk::PipelineCacheCreateFlags::empty())
            .initial_data(&initial_data);

        let cache = unsafe {
            profiling::scope!("vkCreatePipelineCache");
            device.create_pipeline_cache(&cache_info, None)?
        };

        Ok(Self { raw: cache })
    }

    #[allow(dead_code)]
    pub fn save_to_file(&self, device: &Device, path: &Path) -> Result<(), vk::Result> {
        let data = unsafe {
            profiling::scope!("vkGetPipelineCacheData");
            device.get_pipeline_cache_data(self.raw)?
        };

        let mut file = File::create(path).unwrap();
        file.write_all(&data).unwrap();

        Ok(())
    }
}
