use ash::vk;

pub struct Subpass {
    pub color_attachments: Vec<vk::AttachmentReference>,
    pub depth_attachment: Option<vk::AttachmentReference>,
    pub input_attachments: Vec<vk::AttachmentReference>,
    pub bind_point: vk::PipelineBindPoint,
    pub flags: Option<vk::SubpassDescriptionFlags>,
}

impl Subpass {
    pub fn new(bind_point: vk::PipelineBindPoint) -> Self {
        Self {
            color_attachments: vec![],
            depth_attachment: None,
            input_attachments: vec![],
            bind_point,
            flags: None,
        }
    }

    pub fn add_color_attachment_ref(mut self, attachment: vk::AttachmentReference) -> Self {
        self.color_attachments.push(attachment);
        self
    }

    pub fn color_attachments(mut self, attachments: Vec<vk::AttachmentReference>) -> Self {
        self.color_attachments = attachments;
        self
    }

    pub fn add_depth_attachment_ref(mut self, attachment: vk::AttachmentReference) -> Self {
        self.depth_attachment = Some(attachment);
        self
    }

    pub fn add_input_attachment_ref(mut self, attachment: vk::AttachmentReference) -> Self {
        self.input_attachments.push(attachment);
        self
    }

    pub fn input_attachments(mut self, attachments: Vec<vk::AttachmentReference>) -> Self {
        self.input_attachments = attachments;
        self
    }

    pub fn flags(mut self, flags: vk::SubpassDescriptionFlags) -> Self {
        self.flags = Some(flags);
        self
    }
}
