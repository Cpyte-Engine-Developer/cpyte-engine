use vulkanalia::{
    vk::{
        self, ClearColorValue, ClearDepthStencilValue, ClearValue, CommandBufferAllocateInfo,
        CommandBufferBeginInfo, CommandBufferLevel, CommandBufferResetFlags,
        CommandBufferUsageFlags, DescriptorSet, DeviceV1_0, Extent2D, HasBuilder, IndexType,
        Offset2D, PipelineBindPoint, PipelineLayout, Rect2D, RenderPassBeginInfo, ShaderStageFlags,
        SubpassContents, Viewport,
    },
    Device as vkDevice,
};

use crate::{
    buffer::Buffer, command_pool::CommandPool, device::Device, framebuffer::Framebuffer,
    pipeline::Pipeline, render_pass::RenderPass, vertex::Vertex,
};

#[derive(Debug, Clone)]
pub(crate) struct CommandBuffer {
    command_buffer: vk::CommandBuffer,
    device: Device,
    command_pool: CommandPool,
}

impl CommandBuffer {
    pub(crate) fn new(device: Device, command_pool: CommandPool) -> Self {
        let command_buffer_allocate_info = CommandBufferAllocateInfo::builder()
            .command_pool(command_pool.clone().into())
            .level(CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer = unsafe {
            vkDevice::from(device.clone())
                .allocate_command_buffers(&command_buffer_allocate_info)
                .unwrap()[0]
        };

        Self {
            command_buffer,
            device,
            command_pool,
        }
    }

    pub(crate) fn reset(&self) {
        unsafe {
            vkDevice::from(self.device.clone())
                .reset_command_buffer(self.command_buffer, CommandBufferResetFlags::empty())
                .unwrap()
        }
    }

    pub(crate) fn start_recording(
        &self,
        swapchain_extent: Extent2D,
        render_pass: RenderPass,
        framebuffer: Framebuffer,
        pipeline: Pipeline,
    ) {
        let command_buffer_begin_info =
            CommandBufferBeginInfo::builder().flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        let color_clear_value = ClearValue {
            color: ClearColorValue {
                float32: [1.0, 1.0, 1.0, 1.0],
            },
        };

        let depth_clear_value = ClearValue {
            depth_stencil: ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            },
        };

        let clear_values = &[color_clear_value, depth_clear_value];
        let offset = Offset2D::builder().x(0).y(0);
        let render_area = Rect2D::builder().offset(offset).extent(swapchain_extent);

        let render_pass_begin_info = RenderPassBeginInfo::builder()
            .clear_values(clear_values)
            .render_pass(render_pass.render_pass)
            .framebuffer(framebuffer.framebuffer)
            .render_area(render_area);

        let viewports = &[Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(swapchain_extent.width as f32)
            .height(swapchain_extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0)];

        let scissors = &[Rect2D::builder().extent(swapchain_extent).offset(offset)];

        unsafe {
            vkDevice::from(self.device.clone())
                .begin_command_buffer(self.command_buffer, &command_buffer_begin_info)
                .unwrap();

            vkDevice::from(self.device.clone()).cmd_begin_render_pass(
                self.command_buffer,
                &render_pass_begin_info,
                SubpassContents::INLINE,
            );

            vkDevice::from(self.device.clone()).cmd_bind_pipeline(
                self.command_buffer,
                PipelineBindPoint::GRAPHICS,
                pipeline.pipeline,
            );

            vkDevice::from(self.device.clone()).cmd_set_viewport(self.command_buffer, 0, viewports);
            vkDevice::from(self.device.clone()).cmd_set_scissor(self.command_buffer, 0, scissors);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_drawing(
        &self,
        vertex_buffer: Buffer<Vertex>,
        index_buffer: Buffer<u32>,
        pipeline_layout: PipelineLayout,
        descriptor_set: DescriptorSet,
        model_matrix: &[f32],
        indices: &[u32],
    ) {
        unsafe {
            vkDevice::from(self.device.clone()).cmd_bind_vertex_buffers(
                self.command_buffer,
                0,
                &[vertex_buffer.into()],
                &[0],
            );
            vkDevice::from(self.device.clone()).cmd_bind_index_buffer(
                self.command_buffer,
                index_buffer.into(),
                0,
                IndexType::UINT32,
            );

            vkDevice::from(self.device.clone()).cmd_bind_descriptor_sets(
                self.command_buffer,
                PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                0,
                &[descriptor_set],
                &[],
            );
            vkDevice::from(self.device.clone()).cmd_push_constants(
                self.command_buffer,
                pipeline_layout,
                ShaderStageFlags::VERTEX,
                0,
                model_matrix.align_to::<u8>().1,
            );

            vkDevice::from(self.device.clone()).cmd_draw_indexed(
                self.command_buffer,
                indices.len() as u32,
                1,
                0,
                0,
                0,
            );
        }
    }

    pub(crate) fn finish_recording(&self) {
        unsafe {
            vkDevice::from(self.device.clone()).cmd_end_render_pass(self.command_buffer);
            vkDevice::from(self.device.clone())
                .end_command_buffer(self.command_buffer)
                .unwrap();
        }
    }

    pub(crate) fn begin(&self) {
        let command_buffer_begin_info =
            CommandBufferBeginInfo::builder().flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            vkDevice::from(self.device.clone())
                .begin_command_buffer(self.command_buffer, &command_buffer_begin_info)
                .unwrap();
        }
    }

    pub(crate) fn end(&self) {
        unsafe {
            vkDevice::from(self.device.clone())
                .end_command_buffer(self.command_buffer)
                .unwrap();
        }
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            vkDevice::from(self.device.clone())
                .free_command_buffers(self.command_pool.clone().into(), &[self.command_buffer]);
        }
    }
}

impl From<CommandBuffer> for vk::CommandBuffer {
    fn from(value: CommandBuffer) -> Self {
        value.command_buffer
    }
}

impl From<&CommandBuffer> for vk::CommandBuffer {
    fn from(value: &CommandBuffer) -> Self {
        value.command_buffer
    }
}
