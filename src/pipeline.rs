use vulkanalia::{
    vk::{
        self, BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags,
        DescriptorSetLayout, DeviceV1_0, DynamicState, FrontFace, GraphicsPipelineCreateInfo,
        Handle, HasBuilder, LogicOp, PipelineCache, PipelineColorBlendAttachmentState,
        PipelineColorBlendStateCreateInfo, PipelineDepthStencilStateCreateInfo,
        PipelineDynamicStateCreateInfo, PipelineInputAssemblyStateCreateInfo, PipelineLayout,
        PipelineLayoutCreateInfo, PipelineMultisampleStateCreateInfo,
        PipelineRasterizationStateCreateInfo, PipelineShaderStageCreateInfo,
        PipelineVertexInputStateCreateInfo, PipelineViewportStateCreateInfo, PolygonMode,
        PrimitiveTopology, PushConstantRange, SampleCountFlags, ShaderStageFlags,
    },
    Device as vkDevice,
};

use crate::{device::Device, render_pass::RenderPass, shader::Shader, vertex::Vertex};

#[derive(Clone, Debug)]
pub(crate) struct Pipeline {
    pub(crate) pipeline: vk::Pipeline,
    pub(crate) layout: PipelineLayout,
    device: Device,
}

impl Pipeline {
    pub(crate) fn new(
        device: Device,
        descriptor_set_layout: DescriptorSetLayout,
        render_pass: RenderPass,
        msaa_sample_count: SampleCountFlags,
    ) -> Self {
        let layout = Self::create_layout(device.clone(), descriptor_set_layout);
        let pipeline =
            Self::create_pipeline(device.clone(), layout, render_pass, msaa_sample_count);

        Self {
            pipeline,
            layout,
            device,
        }
    }

    fn create_pipeline(
        device: Device,
        pipeline_layout: PipelineLayout,
        render_pass: RenderPass,
        msaa_sample_count: SampleCountFlags,
    ) -> vk::Pipeline {
        let vertex_shader_bytes = include_bytes!("../shaders/build/main.vert.spv");
        let fragment_shader_bytes = include_bytes!("../shaders/build/main.frag.spv");

        let vertex_shader = Shader::new(device.clone(), vertex_shader_bytes);
        let fragment_shader = Shader::new(device.clone(), fragment_shader_bytes);

        let vertex_shader_stage_create_info = PipelineShaderStageCreateInfo::builder()
            .stage(ShaderStageFlags::VERTEX)
            .module(vertex_shader.module)
            .name(b"main\0");
        let fragment_shader_stage_create_info = PipelineShaderStageCreateInfo::builder()
            .stage(ShaderStageFlags::FRAGMENT)
            .module(fragment_shader.module)
            .name(b"main\0");

        let binding_descriptions = &[Vertex::binding_description()];
        let attribute_descriptions = Vertex::attribute_descriptions();
        let vertex_input_create_info = PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions);

        let input_assembly_create_info = PipelineInputAssemblyStateCreateInfo::builder()
            .topology(PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport_create_info = PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1);

        let rasterization_create_info = PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(CullModeFlags::BACK)
            .front_face(FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false);

        let multisample_create_info = PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(msaa_sample_count != SampleCountFlags::_1)
            .min_sample_shading(0.2)
            .rasterization_samples(msaa_sample_count);

        let depth_stencil_create_info = PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0)
            .stencil_test_enable(false);
        // .front(StencilOpState::builder())
        // .back(StencilOpState::builder())

        let color_blend_attachment_create_info = PipelineColorBlendAttachmentState::builder()
            .color_write_mask(ColorComponentFlags::all())
            .blend_enable(true)
            .src_color_blend_factor(BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(BlendOp::ADD)
            .src_alpha_blend_factor(BlendFactor::ONE)
            .dst_alpha_blend_factor(BlendFactor::ZERO)
            .alpha_blend_op(BlendOp::ADD);

        let color_blend_attachment_create_infos = &[color_blend_attachment_create_info];
        let color_blend_create_info = PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(LogicOp::COPY)
            .attachments(color_blend_attachment_create_infos)
            .blend_constants([0.0; 4]);

        let dynamic_states = &[DynamicState::VIEWPORT, DynamicState::SCISSOR];

        let dynamic_state_create_info =
            PipelineDynamicStateCreateInfo::builder().dynamic_states(dynamic_states);

        let pipeline_stages = &[
            vertex_shader_stage_create_info,
            fragment_shader_stage_create_info,
        ];

        let graphics_pipeline_create_info = GraphicsPipelineCreateInfo::builder()
            .stages(pipeline_stages)
            .vertex_input_state(&vertex_input_create_info)
            .input_assembly_state(&input_assembly_create_info)
            .viewport_state(&viewport_create_info)
            .dynamic_state(&dynamic_state_create_info)
            .rasterization_state(&rasterization_create_info)
            .multisample_state(&multisample_create_info)
            .depth_stencil_state(&depth_stencil_create_info)
            .color_blend_state(&color_blend_create_info)
            .layout(pipeline_layout)
            .render_pass(render_pass.render_pass)
            .subpass(0)
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(-1);

        let graphics_pipeline = unsafe {
            vkDevice::from(device)
                .create_graphics_pipelines(
                    PipelineCache::null(),
                    &[graphics_pipeline_create_info],
                    None,
                )
                .unwrap()
                .0
        };

        vertex_shader.destroy();
        fragment_shader.destroy();

        graphics_pipeline
    }

    fn create_layout(device: Device, descriptor_set_layout: DescriptorSetLayout) -> PipelineLayout {
        let vert_push_constant_range = PushConstantRange::builder()
            .stage_flags(ShaderStageFlags::VERTEX)
            .offset(0)
            .size(64);

        let push_constant_ranges = &[vert_push_constant_range];
        let descriptor_set_layouts = &[descriptor_set_layout];
        let layout_create_info = PipelineLayoutCreateInfo::builder()
            .set_layouts(descriptor_set_layouts)
            .push_constant_ranges(push_constant_ranges);

        unsafe {
            vkDevice::from(device)
                .create_pipeline_layout(&layout_create_info, None)
                .unwrap()
        }
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            vkDevice::from(self.device.clone()).destroy_pipeline(self.pipeline, None);
            vkDevice::from(self.device.clone()).destroy_pipeline_layout(self.layout, None);
        }
    }
}

impl From<Pipeline> for vk::Pipeline {
    fn from(value: Pipeline) -> Self {
        value.pipeline
    }
}

impl From<&Pipeline> for vk::Pipeline {
    fn from(value: &Pipeline) -> Self {
        value.pipeline
    }
}
