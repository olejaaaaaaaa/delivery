use ash::vk::{self, PipelineShaderStageCreateInfo};
use tracing::debug;
use crate::core::{Device, VulkanResult};

pub struct ComputePipeline {
    pub raw: vk::Pipeline,
}

pub struct ComputePipelineBuilder<'n> {
    device: &'n Device,
    flags: Option<vk::PipelineCreateFlags>,
    pipeline_layout: Option<vk::PipelineLayout>,
    shader: Option<vk::ShaderModule>,
}

impl<'n> ComputePipelineBuilder<'n> {
    pub fn new(device: &'n Device) -> Self {
        Self {
            device,
            shader: None,
            flags: None,
            pipeline_layout: None,
        }
    }

    pub fn pipeline_layout(mut self, layout: vk::PipelineLayout) -> Self {
        self.pipeline_layout = Some(layout);
        self
    }

    
    pub fn shader(mut self, module: vk::ShaderModule) -> Self {
        self.shader = Some(module);
        self
    }

    pub fn build(self) -> VulkanResult<ComputePipeline> {

        let shader = self.shader.unwrap();

        let stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .name(c"main")
            .module(shader);

        let layout = self.pipeline_layout.unwrap();
        let flags = self.flags.unwrap_or(vk::PipelineCreateFlags::empty());

        let create_info = vk::ComputePipelineCreateInfo::default()
            .layout(layout)
            .stage(stage)
            .flags(flags);

        let pipeline = unsafe { self.device.create_compute_pipelines(
            vk::PipelineCache::null(), 
            &[create_info], 
            None
        ).unwrap() }[0];

        debug!("Create Compute Pipeline: {:#?}", create_info);

        Ok(ComputePipeline { 
            raw: pipeline 
        })
    }
}
