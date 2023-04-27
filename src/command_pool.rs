use vulkanalia::{
    vk::{self, CommandPoolCreateFlags, CommandPoolCreateInfo, DeviceV1_0, HasBuilder},
    Device as vkDevice,
};

use crate::device::Device;

#[derive(Debug, Clone)]
pub(crate) struct CommandPool {
    command_pool: vk::CommandPool,
    device: Device,
}

impl CommandPool {
    pub(crate) fn new(device: Device, graphics_queue_family_index: u32) -> Self {
        let command_pool_create_info = CommandPoolCreateInfo::builder()
            .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(graphics_queue_family_index);

        let command_pool = unsafe {
            vkDevice::from(device.clone())
                .create_command_pool(&command_pool_create_info, None)
                .unwrap()
        };

        Self {
            command_pool,
            device,
        }
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            vkDevice::from(self.device.clone()).destroy_command_pool(self.command_pool, None);
        }
    }
}

impl From<CommandPool> for vk::CommandPool {
    fn from(value: CommandPool) -> Self {
        value.command_pool
    }
}

impl From<&CommandPool> for vk::CommandPool {
    fn from(value: &CommandPool) -> Self {
        value.command_pool
    }
}
