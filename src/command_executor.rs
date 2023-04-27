use vulkanalia::vk::{HasBuilder, SubmitInfo};

use crate::{
    command_buffer::CommandBuffer, command_pool::CommandPool, device::Device, fence::Fence,
    queue::Queue,
};

pub(crate) struct CommandExecutor;

impl CommandExecutor {
    pub(crate) fn execute<F: FnOnce(CommandBuffer)>(
        command_pool: CommandPool,
        device: Device,
        command: F,
        graphics_queue: Queue,
    ) {
        let command_buffer = CommandBuffer::new(device.clone(), command_pool);

        command_buffer.begin();
        command(command_buffer.clone());
        command_buffer.end();

        let command_buffers = &[command_buffer.clone().into()];
        let submit_info = SubmitInfo::builder()
            .command_buffers(command_buffers)
            .build();
        let fence = Fence::new(device, false);

        graphics_queue.submit(submit_info, fence.clone());
        graphics_queue.wait_idle();

        fence.destroy();
        command_buffer.destroy();
    }
}
