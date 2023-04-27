use vulkanalia::{
    vk::{self, DeviceV1_0, KhrSwapchainExtension, PresentInfoKHR, SubmitInfo},
    Device as vkDevice,
};

use crate::{device::Device, fence::Fence};

#[derive(Debug, Clone)]
pub(crate) struct Queue {
    pub(crate) queue: vk::Queue,
    device: Device,
}

impl Queue {
    pub(crate) fn new(device: Device, queue_family_index: u32) -> Self {
        let queue =
            unsafe { vkDevice::from(device.clone()).get_device_queue(queue_family_index, 0) };

        Self { queue, device }
    }

    pub(crate) fn submit(&self, submit_info: SubmitInfo, signaled_fence: Fence) {
        unsafe {
            vkDevice::from(self.device.clone())
                .queue_submit(self.queue, &[submit_info], signaled_fence.fence)
                .unwrap();
        }
    }

    pub(crate) fn present(&self, present_info: PresentInfoKHR) {
        unsafe {
            vkDevice::from(self.device.clone())
                .queue_present_khr(self.queue, &present_info)
                .unwrap();
        }
    }

    pub(crate) fn wait_idle(&self) {
        unsafe {
            vkDevice::from(self.device.clone())
                .queue_wait_idle(self.queue)
                .unwrap();
        }
    }
}

impl From<Queue> for vk::Queue {
    fn from(value: Queue) -> Self {
        value.queue
    }
}

impl From<&Queue> for vk::Queue {
    fn from(value: &Queue) -> Self {
        value.queue
    }
}
