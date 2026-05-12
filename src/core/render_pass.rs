use ash::vk;
use tracing::debug;

use super::device::Device;
use super::{Subpass, VulkanError, VulkanResult};

pub struct RenderPass {
    pub(crate) raw: vk::RenderPass,
}

impl RenderPass {
    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_render_pass(self.raw, None);
        }
    }
}

pub struct RenderPassBuilder<'a> {
    pub device: &'a Device,
    pub attachments: Vec<vk::AttachmentDescription>,
    pub dependencies: Vec<vk::SubpassDependency>,
    pub subpasses: Vec<Subpass>,
}

impl<'a> RenderPassBuilder<'a> {
    pub fn default(device: &'a Device, color: vk::Format, depth: vk::Format) -> Self {
        let subpass = Subpass::new(vk::PipelineBindPoint::GRAPHICS)
            .add_color_attachment_ref(
                vk::AttachmentReference::default()
                    .attachment(0)
                    .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL),
            )
            .add_depth_attachment_ref(
                vk::AttachmentReference::default()
                    .attachment(1)
                    .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
            );

        RenderPassBuilder {
            device,
            attachments: vec![
                vk::AttachmentDescription {
                    flags: vk::AttachmentDescriptionFlags::empty(),
                    format: color,
                    samples: vk::SampleCountFlags::TYPE_1,
                    load_op: vk::AttachmentLoadOp::CLEAR,
                    store_op: vk::AttachmentStoreOp::STORE,
                    stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                    stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                    initial_layout: vk::ImageLayout::UNDEFINED,
                    final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                },
                vk::AttachmentDescription::default()
                    .format(depth)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
            ],
            dependencies: vec![vk::SubpassDependency {
                src_subpass: vk::SUBPASS_EXTERNAL,
                dst_subpass: 0,
                src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                src_access_mask: vk::AccessFlags::empty(),
                dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                dependency_flags: vk::DependencyFlags::BY_REGION,
            }],
            subpasses: vec![subpass],
        }
    }

    pub fn new(device: &'a Device) -> Self {
        Self {
            device,
            attachments: vec![],
            dependencies: vec![],
            subpasses: vec![],
        }
    }

    pub fn dependencies(mut self, dependencies: Vec<vk::SubpassDependency>) -> Self {
        self.dependencies = dependencies;
        self
    }

    pub fn attachments(mut self, attachments: Vec<vk::AttachmentDescription>) -> Self {
        self.attachments = attachments;
        self
    }

    pub fn subpasses(mut self, subpasses: Vec<Subpass>) -> Self {
        self.subpasses = subpasses;
        self
    }

    pub fn build(self) -> VulkanResult<RenderPass> {
        let device = self.device;
        let mut subpasses = vec![];

        for i in &self.subpasses {
            let mut subpass = vk::SubpassDescription::default()
                .color_attachments(&i.color_attachments)
                .flags(i.flags.unwrap_or(vk::SubpassDescriptionFlags::empty()))
                .input_attachments(&i.input_attachments)
                .pipeline_bind_point(i.bind_point);

            if let Some(depth) = &i.depth_attachment {
                subpass = subpass.depth_stencil_attachment(depth);
            }

            subpasses.push(subpass);
        }

        let attachment_desc = self.attachments;
        let subpass_dep = self.dependencies;

        debug_assert!(!subpass_dep.is_empty(), "Subpass is empty");
        debug_assert!(!attachment_desc.is_empty(), "Attachments is empty");

        let create_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachment_desc)
            .dependencies(&subpass_dep)
            .subpasses(&subpasses);

        let render_pass = unsafe {
            profiling::scope!("vkCreateRenderPass");
            device
                .create_render_pass(&create_info, None)
                .map_err(VulkanError::Unknown)?
        };

        debug!("Render Pass: {:#?}", create_info);

        Ok(RenderPass { raw: render_pass })
    }
}
