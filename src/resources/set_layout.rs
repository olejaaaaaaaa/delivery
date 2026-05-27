use std::collections::HashMap;

use ash::vk;

use crate::core::{DescriptorSetLayout, DescriptorSetLayoutBuilder, Device, VulkanResult};

#[derive(Clone)]
pub struct SetLayoutDesc {
    pub bindings: Vec<vk::DescriptorSetLayoutBinding<'static>>
}

impl Eq for SetLayoutDesc {}

impl std::hash::Hash for SetLayoutDesc {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for i in &self.bindings {
            i.binding.hash(state);
            i.descriptor_count.hash(state);
            i.descriptor_type.hash(state);
            i.stage_flags.hash(state);
        }
    }
}

impl PartialEq for SetLayoutDesc {
    fn eq(&self, other: &Self) -> bool {
        for (index, binding) in self.bindings.iter().enumerate() {
            if let Some(other_binding) = other.bindings.get(index) {

                if !(binding.binding == other_binding.binding) {
                    return false;
                }

                if !(binding.descriptor_count == other_binding.descriptor_count) {
                    return false;
                }

                if !(binding.descriptor_type == other_binding.descriptor_type) {
                    return false;
                }

                if !(binding.stage_flags == other_binding.stage_flags) {
                    return false;
                }

            } else {
                return false;
            }
        }
        return true;
    }
}

pub struct SetLayoutCache {
    cache: HashMap<SetLayoutDesc, DescriptorSetLayout>
}

impl SetLayoutCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new()
        }
    }
}

impl SetLayoutCache {
   pub fn get_or_create(&mut self, device: &Device, desc: SetLayoutDesc) -> VulkanResult<&DescriptorSetLayout> {
        if !self.cache.contains_key(&desc) {
            let set_layout = DescriptorSetLayoutBuilder::new(device)
                .bindings(&desc.bindings)
                .build()?;
            self.cache.insert(desc.clone(), set_layout);
        }
        Ok(self.cache.get(&desc).unwrap())
    }
}