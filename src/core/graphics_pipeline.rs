use ash::vk;
use tracing::debug;

use super::device::Device;
use super::VulkanResult;

pub struct GraphicsPipeline {
    pub raw: vk::Pipeline,
}

pub struct GraphicsPipelineBuilder<'n> {
    device: &'n Device,
    pipeline_layout: Option<vk::PipelineLayout>,
    cache: Option<vk::PipelineCache>,
    render_pass: Option<vk::RenderPass>,
    depth_stencil: Option<vk::PipelineDepthStencilStateCreateInfo<'n>>,
    descriptor_set_layout: Option<&'n [vk::DescriptorSetLayout]>,
    color_blending_info: Option<vk::PipelineColorBlendStateCreateInfo<'n>>,
    vertex_input_info: Option<vk::PipelineVertexInputStateCreateInfo<'n>>,
    input_assembly_info: Option<vk::PipelineInputAssemblyStateCreateInfo<'n>>,
    multisampling_info: Option<vk::PipelineMultisampleStateCreateInfo<'n>>,
    rasterization: Option<vk::PipelineRasterizationStateCreateInfo<'n>>,
    viewport: Option<Vec<vk::Viewport>>,
    scissors: Option<Vec<vk::Rect2D>>,
    dynamic_state: Option<Vec<vk::DynamicState>>,
    vertex_shader: Option<vk::ShaderModule>,
    fragment_shader: Option<vk::ShaderModule>,
}

impl<'n> GraphicsPipelineBuilder<'n> {
    pub fn new(device: &'n Device) -> Self {
        Self {
            device,
            cache: None,
            pipeline_layout: None,
            depth_stencil: None,
            render_pass: None,
            descriptor_set_layout: None,
            color_blending_info: None,
            vertex_input_info: None,
            input_assembly_info: None,
            multisampling_info: None,
            rasterization: None,
            fragment_shader: None,
            vertex_shader: None,
            scissors: None,
            viewport: None,
            dynamic_state: None,
        }
    }

    #[allow(dead_code)]
    pub fn cache(mut self, cache: vk::PipelineCache) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn rasterization(
        mut self,
        rasterization: vk::PipelineRasterizationStateCreateInfo<'static>,
    ) -> Self {
        self.rasterization = Some(rasterization);
        self
    }

    pub fn multisampling(
        mut self,
        multisampling: vk::PipelineMultisampleStateCreateInfo<'static>,
    ) -> Self {
        self.multisampling_info = Some(multisampling);
        self
    }

    pub fn color_blending(
        mut self,
        color_blending: vk::PipelineColorBlendStateCreateInfo<'n>,
    ) -> Self {
        self.color_blending_info = Some(color_blending);
        self
    }

    pub fn scissors(mut self, scissors: Vec<vk::Rect2D>) -> Self {
        self.scissors = Some(scissors);
        self
    }

    #[allow(dead_code)]
    pub fn descriptor_set_layout(mut self, layouts: &'n [vk::DescriptorSetLayout]) -> Self {
        self.descriptor_set_layout = Some(layouts);
        self
    }

    pub fn vertex_input_info(mut self, info: vk::PipelineVertexInputStateCreateInfo<'n>) -> Self {
        self.vertex_input_info = Some(info);
        self
    }

    pub fn pipeline_layout(mut self, layout: vk::PipelineLayout) -> Self {
        self.pipeline_layout = Some(layout);
        self
    }

    pub fn render_pass(mut self, render_pass: vk::RenderPass) -> Self {
        self.render_pass = Some(render_pass);
        self
    }

    pub fn vertex_shader(mut self, module: vk::ShaderModule) -> Self {
        self.vertex_shader = Some(module);
        self
    }

    pub fn fragment_shader(mut self, module: vk::ShaderModule) -> Self {
        self.fragment_shader = Some(module);
        self
    }

    pub fn dynamic_state(mut self, state: Vec<vk::DynamicState>) -> Self {
        self.dynamic_state = Some(state);
        self
    }

    pub fn depth_stencil(mut self, depth_stencil: vk::PipelineDepthStencilStateCreateInfo<'n>) -> Self {
        self.depth_stencil= Some(depth_stencil);
        self
    }

    pub fn input_assembly(
        mut self,
        input_assembly: vk::PipelineInputAssemblyStateCreateInfo<'static>,
    ) -> Self {
        self.input_assembly_info = Some(input_assembly);
        self
    }

    pub fn viewport(mut self, viewport: Vec<vk::Viewport>) -> Self {
        self.viewport = Some(viewport);
        self
    }

    pub fn build(self) -> VulkanResult<GraphicsPipeline> {
        let mut create_info = vk::GraphicsPipelineCreateInfo::default();

        // ------------- Dynamic State ------------------------
        
        let dynamic_states = self.dynamic_state.as_deref().unwrap_or(&[]);

        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::default()
                .dynamic_states(dynamic_states);

        create_info = create_info.dynamic_state(&dynamic_state_info);
        // ----------------- End ------------------------------------

        // ----------------- Shader States
        // -------------------------------
        let mut shader_states_infos = vec![];

        if let Some(vertex) = self.vertex_shader {
            shader_states_infos.push(
                vk::PipelineShaderStageCreateInfo::default()
                    .module(vertex)
                    .name(c"main")
                    .stage(vk::ShaderStageFlags::VERTEX),
            );
        }

        if let Some(fragment) = self.fragment_shader {
            shader_states_infos.push(
                vk::PipelineShaderStageCreateInfo::default()
                    .module(fragment)
                    .name(c"main")
                    .stage(vk::ShaderStageFlags::FRAGMENT),
            );
        }

        create_info = create_info.stages(&shader_states_infos);
        // ----------------- End ------------------------------------

        // --------------- Viewport and Scissors -------------------
        let viewport = self.viewport.expect("Viewport");
        let scissors = self.scissors.expect("Scissors");

        let viewport_info = vk::PipelineViewportStateCreateInfo::default()
            .viewports(&viewport)
            .scissors(&scissors);

        create_info = create_info.viewport_state(&viewport_info);
        // ----------------- End ------------------------------------

        let vertex_input_info = self.vertex_input_info.unwrap();
        let input_assembly_info = self.input_assembly_info.unwrap();
        let raster = self.rasterization.unwrap();
        let multisampling = self.multisampling_info.unwrap();
        let color_blend = self.color_blending_info.unwrap();
        let layout = self.pipeline_layout.unwrap();
        let render_pass = self.render_pass.unwrap();

        let depth_stencil_state = self.depth_stencil.unwrap_or(
            vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0)
            .stencil_test_enable(false)
        );

        create_info = create_info
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .rasterization_state(&raster)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blend)
            .depth_stencil_state(&depth_stencil_state)
            .layout(layout)
            .render_pass(render_pass);

        let pipeline = unsafe {
            profiling::scope!("vkCreateGraphicsPipelines");
            self.device
                .create_graphics_pipelines(
                    self.cache.unwrap_or(vk::PipelineCache::null()),
                    &[create_info],
                    None,
                )
                .expect("Error create Graphics Pipeline")[0]
        };

        debug!("Create Graphics Pipeline: {:#?}", create_info);

        for i in shader_states_infos {
            unsafe { self.device.destroy_shader_module(i.module, None) };
        }

        Ok(GraphicsPipeline { raw: pipeline })
    }
}
