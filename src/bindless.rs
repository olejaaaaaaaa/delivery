use std::sync::{Arc, atomic::{AtomicU32, Ordering}};

use ash::vk;

use crate::{core::{DescriptorPool, DescriptorPoolBuilder, DescriptorSetLayout, DescriptorSetLayoutBuilder, Device, VulkanResult}, render_context::RenderContext};


const MAX_SAMPLED_IMAGE: u32 = 16_384;
const MAX_STORAGE_IMAGES: u32 = 1_024;
const MAX_SAMPLER: u32 = 5;

pub(crate) struct Bindless {
    pub(crate) set_layout: DescriptorSetLayout,
    pub(crate) set: vk::DescriptorSet,
    pub(crate) pool: DescriptorPool,
    pub(crate) next_texture: AtomicU32,
}

impl Bindless {
    pub fn new(ctx: &Arc<RenderContext>) -> VulkanResult<Self> {
        let _limits = ctx.device.props2.properties.limits;

        let layout = vec![
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                .descriptor_count(MAX_SAMPLED_IMAGE)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        ];

        let binding_flags = vec![
            vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                | vk::DescriptorBindingFlags::PARTIALLY_BOUND
                | vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING,
            vk::DescriptorBindingFlags::empty(),
        ];

        let mut binding_flags_info =
            vk::DescriptorSetLayoutBindingFlagsCreateInfo::default().binding_flags(&binding_flags);

        let set_layout = DescriptorSetLayoutBuilder::new(&ctx.device)
            .bindings(layout.clone())
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
            .push_next(&mut binding_flags_info)
            .build()?;

        let mut pool_sizes = vec![];

        for i in layout.iter() {
            pool_sizes.push(
                vk::DescriptorPoolSize::default()
                    .descriptor_count(i.descriptor_count)
                    .ty(i.descriptor_type),
            );
        }

        let pool = DescriptorPoolBuilder::new(&ctx.device)
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
            .pool_sizes(&pool_sizes)
            .max_sets(1)
            .build()?;

        let layouts = [set_layout.raw];
        let set = pool.create_descriptor_set(&ctx.device, &layouts)?[0];

        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(0.0);

        let sampler = unsafe { ctx.device.create_sampler(&sampler_info, None).unwrap() };

        let sampler_info_desc = vk::DescriptorImageInfo::default().sampler(sampler);

        let sampler_write = vk::WriteDescriptorSet::default()
            .dst_set(set)
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::SAMPLER)
            .image_info(std::slice::from_ref(&sampler_info_desc));

        unsafe { ctx.device.update_descriptor_sets(&[sampler_write], &[]) };

        Ok(Self {
            next_texture: AtomicU32::new(0),
            set_layout,
            set,
            pool,
        })
    }

    pub fn alloc_texture(&self, device: &Device, image_view: vk::ImageView) -> u32 {
        let index = self.next_texture.fetch_add(1, Ordering::Relaxed);
        self.update_texture(device, index, image_view);
        index
    }

    pub fn update_texture(&self, device: &Device, index: u32, image_view: vk::ImageView) {
        let image_info = vk::DescriptorImageInfo::default()
            .image_view(image_view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

        let write = vk::WriteDescriptorSet::default()
            .dst_set(self.set)
            .dst_binding(0)
            .dst_array_element(index)
            .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
            .image_info(std::slice::from_ref(&image_info));

        println!("New bindless index: {}", index);

        unsafe { device.update_descriptor_sets(&[write], &[]) };
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_descriptor_pool(self.pool.raw, None);
            device.destroy_descriptor_set_layout(self.set_layout.raw, None);
        }
    }
}
