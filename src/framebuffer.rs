use vulkanalia::{
    vk::{self, DeviceV1_0, Extent2D, FramebufferCreateInfo, HasBuilder, ImageView},
    Device as vkDevice,
};

use crate::{device::Device, render_pass::RenderPass};

#[derive(Debug, Clone)]
pub(crate) struct Framebuffer {
    pub(crate) framebuffer: vk::Framebuffer,
    device: Device,
}

impl Framebuffer {
    pub(crate) fn new(
        device: Device,
        swapchain_image_view: &ImageView,
        render_pass: RenderPass,
        swapchain_extent: Extent2D,
        depth_image_view: ImageView,
        color_image_view: Option<ImageView>,
    ) -> Self {
        let attachments = if color_image_view.is_some() {
            vec![
                color_image_view.unwrap(),
                depth_image_view,
                *swapchain_image_view,
            ]
        } else {
            vec![*swapchain_image_view, depth_image_view]
        };

        let framebuffer_create_info = FramebufferCreateInfo::builder()
            .render_pass(render_pass.render_pass)
            .attachments(attachments.as_slice())
            .width(swapchain_extent.width)
            .height(swapchain_extent.height)
            .layers(1);

        let framebuffer = unsafe {
            vkDevice::from(device.clone())
                .create_framebuffer(&framebuffer_create_info, None)
                .unwrap()
        };

        Self {
            framebuffer,
            device,
        }
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            vkDevice::from(self.device.clone()).destroy_framebuffer(self.framebuffer, None);
        }
    }
}

impl From<Framebuffer> for vk::Framebuffer {
    fn from(value: Framebuffer) -> Self {
        value.framebuffer
    }
}

impl From<&Framebuffer> for vk::Framebuffer {
    fn from(value: &Framebuffer) -> Self {
        value.framebuffer
    }
}
