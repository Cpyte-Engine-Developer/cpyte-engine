use vulkanalia::{
    vk::{self, DeviceV1_0, FenceCreateFlags, FenceCreateInfo, Handle, HasBuilder},
    Device as vkDevice,
};

use crate::device::Device;

#[derive(Debug, Clone)]
pub(crate) struct Fence {
    pub(crate) fence: vk::Fence,
    device: Device,
}

impl Fence {
    pub(crate) fn new(device: Device, signaled: bool) -> Self {
        let fence = match signaled {
            true => {
                let fence_create_info =
                    FenceCreateInfo::builder().flags(FenceCreateFlags::SIGNALED);

                unsafe {
                    vkDevice::from(device.clone())
                        .create_fence(&fence_create_info, None)
                        .unwrap()
                }
            }
            false => vk::Fence::null(),
        };

        Self { fence, device }
    }

    pub(crate) fn wait(&self) {
        unsafe {
            vkDevice::from(self.device.clone())
                .wait_for_fences(&[self.fence], true, u64::MAX)
                .unwrap();
        }
    }

    pub(crate) fn reset(&self) {
        unsafe {
            vkDevice::from(self.device.clone())
                .reset_fences(&[self.fence])
                .unwrap();
        }
    }

    pub(crate) fn is_null(&self) -> bool {
        self.fence.is_null()
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            vkDevice::from(self.device.clone()).destroy_fence(self.fence, None);
        }
    }
}

impl From<Fence> for vk::Fence {
    fn from(value: Fence) -> Self {
        value.fence
    }
}

impl From<&Fence> for vk::Fence {
    fn from(value: &Fence) -> Self {
        value.fence
    }
}
