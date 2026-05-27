
mod set_layout;
use std::sync::Arc;

use ash::vk;
use parking_lot::Mutex;
pub use set_layout::*;

use crate::{Bindless, RenderContext, core::{DescriptorPool, DescriptorPoolBuilder, DescriptorSetLayout, Device, VulkanResult}};

pub struct Resources {
    pub(crate) set_layout: Mutex<SetLayoutCache>,
    pub(crate) pool: DescriptorPool,
    pub(crate) bindless: Arc<Bindless>,
}

impl Resources {

    // pub fn set_layout(&self, device: &Device, desc: SetLayoutDesc) -> VulkanResult<&DescriptorSetLayout> {
    //     let mut cache = self.set_layout.lock();
    //     cache.get_or_create(device, desc)
    // }

    pub fn new(ctx: &Arc<RenderContext>) -> VulkanResult<Arc<Self>> {

        let bindless = Bindless::new(&ctx)?;

        let pool = DescriptorPoolBuilder::new(&ctx.device)
            .max_sets(50)
            .pool_sizes(&[
                vk::DescriptorPoolSize::default()
                    .descriptor_count(5)
                    .ty(vk::DescriptorType::SAMPLED_IMAGE),
                vk::DescriptorPoolSize::default()
                    .descriptor_count(25)
                    .ty(vk::DescriptorType::STORAGE_BUFFER),
                vk::DescriptorPoolSize::default()
                    .descriptor_count(5)
                    .ty(vk::DescriptorType::STORAGE_IMAGE),
                vk::DescriptorPoolSize::default()
                    .descriptor_count(15)
                    .ty(vk::DescriptorType::UNIFORM_BUFFER),
            ])
            .build()?;

        Ok(Arc::new(Self {
            set_layout: Mutex::new(SetLayoutCache::new()),
            bindless: Arc::new(bindless),
            pool
        }))
    }
}