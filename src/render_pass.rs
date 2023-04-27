use vulkanalia::{
    vk::{
        self, AccessFlags, AttachmentDescription, AttachmentLoadOp, AttachmentReference,
        AttachmentStoreOp, DeviceV1_0, Format, HasBuilder, ImageLayout, PipelineBindPoint,
        PipelineStageFlags, RenderPassCreateInfo, SampleCountFlags, SubpassDependency,
        SubpassDescription, SUBPASS_EXTERNAL,
    },
    Device as vkDevice,
};

use crate::device::Device;

#[derive(Debug, Clone)]
pub(crate) struct RenderPass {
    pub(crate) render_pass: vk::RenderPass,
    device: Device,
}

impl RenderPass {
    pub(crate) fn new(
        device: Device,
        color_attachment_format: Format,
        depth_attachment_format: Format,
        swapchain_format: Format,
        msaa_sample_count: SampleCountFlags,
    ) -> Self {
        let color_attachment_description = AttachmentDescription::builder()
            .format(color_attachment_format)
            .samples(msaa_sample_count)
            .load_op(AttachmentLoadOp::CLEAR)
            .store_op(AttachmentStoreOp::STORE)
            .stencil_load_op(AttachmentLoadOp::DONT_CARE)
            .stencil_load_op(AttachmentLoadOp::DONT_CARE)
            .initial_layout(ImageLayout::UNDEFINED)
            .final_layout(if msaa_sample_count != SampleCountFlags::_1 {
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            } else {
                ImageLayout::PRESENT_SRC_KHR
            });
        let depth_attachment_description = AttachmentDescription::builder()
            .format(depth_attachment_format)
            .samples(msaa_sample_count)
            .load_op(AttachmentLoadOp::CLEAR)
            .store_op(AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(AttachmentStoreOp::DONT_CARE)
            .initial_layout(ImageLayout::UNDEFINED)
            .final_layout(ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let attachment_descriptions = if msaa_sample_count != SampleCountFlags::_1 {
            let color_resolve_attachment_description = AttachmentDescription::builder()
                .format(swapchain_format)
                .samples(SampleCountFlags::_1)
                .load_op(AttachmentLoadOp::DONT_CARE)
                .store_op(AttachmentStoreOp::STORE)
                .stencil_load_op(AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(AttachmentStoreOp::DONT_CARE)
                .initial_layout(ImageLayout::UNDEFINED)
                .final_layout(ImageLayout::PRESENT_SRC_KHR);

            vec![
                color_attachment_description,
                depth_attachment_description,
                color_resolve_attachment_description,
            ]
        } else {
            vec![color_attachment_description, depth_attachment_description]
        };

        let color_attachment_ref = AttachmentReference::builder()
            .attachment(0)
            .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let depth_stencil_attachment_ref = AttachmentReference::builder()
            .attachment(1)
            .layout(ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
        let color_attachment_refs = &[color_attachment_ref];

        let color_resolve_attachment_refs = if msaa_sample_count != SampleCountFlags::_1 {
            let color_resolve_attachment_ref = AttachmentReference::builder()
                .attachment(2)
                .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .build();

            vec![color_resolve_attachment_ref]
        } else {
            vec![]
        };

        let subpass_dependency = SubpassDependency::builder()
            .src_subpass(SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .src_access_mask(AccessFlags::empty())
            .dst_stage_mask(
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .dst_access_mask(
                AccessFlags::COLOR_ATTACHMENT_WRITE | AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            );

        let subpass_dependencies = &[subpass_dependency];

        let mut subpass_description = SubpassDescription::builder()
            .pipeline_bind_point(PipelineBindPoint::GRAPHICS)
            .color_attachments(color_attachment_refs)
            .depth_stencil_attachment(&depth_stencil_attachment_ref);

        if msaa_sample_count != SampleCountFlags::_1 {
            subpass_description.resolve_attachments = color_resolve_attachment_refs.as_ptr();
        }

        let subpasses = &[subpass_description];

        let render_pass_create_info = RenderPassCreateInfo::builder()
            .attachments(attachment_descriptions.as_slice())
            .subpasses(subpasses)
            .dependencies(subpass_dependencies);

        let render_pass = unsafe {
            vkDevice::from(device.clone())
                .create_render_pass(&render_pass_create_info, None)
                .unwrap()
        };

        Self {
            render_pass,
            device,
        }
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            vkDevice::from(self.device.clone()).destroy_render_pass(self.render_pass, None);
        }
    }
}

impl From<RenderPass> for vk::RenderPass {
    fn from(value: RenderPass) -> Self {
        value.render_pass
    }
}

impl From<&RenderPass> for vk::RenderPass {
    fn from(value: &RenderPass) -> Self {
        value.render_pass
    }
}
