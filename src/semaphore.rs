use vulkanalia::{
    vk::{self, DeviceV1_0, HasBuilder, SemaphoreCreateInfo},
    Device as vkDevice,
};

use crate::device::Device;

#[derive(Debug, Clone)]
pub(crate) struct Semaphore {
    pub(crate) semaphore: vk::Semaphore,
    device: Device,
}

impl Semaphore {
    pub(crate) fn new(device: Device) -> Self {
        let semaphore_create_info = SemaphoreCreateInfo::builder();

        let semaphore = unsafe {
            vkDevice::from(device.clone())
                .create_semaphore(&semaphore_create_info, None)
                .unwrap()
        };

        Self { semaphore, device }
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            vkDevice::from(self.device.clone()).destroy_semaphore(self.semaphore, None);
        }
    }
}

impl From<Semaphore> for vk::Semaphore {
    fn from(value: Semaphore) -> Self {
        value.semaphore
    }
}

impl From<&Semaphore> for vk::Semaphore {
    fn from(value: &Semaphore) -> Self {
        value.semaphore
    }
}
