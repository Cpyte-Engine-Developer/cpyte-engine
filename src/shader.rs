use vulkanalia::{
    vk::{self, DeviceV1_0, HasBuilder, ShaderModuleCreateInfo},
    Device as vkDevice,
};

use log::error;

use crate::device::Device;

#[derive(Debug, Clone)]
pub(crate) struct Shader {
    pub(crate) module: vk::ShaderModule,
    device: Device,
}

impl Shader {
    pub(crate) fn new(device: Device, bytes: &[u8]) -> Self {
        let module = Self::create_module(device.clone(), bytes);

        Self { module, device }
    }

    fn create_module(device: Device, bytes: &[u8]) -> vk::ShaderModule {
        let byte_vector = Vec::<u8>::from(bytes);
        let (prefix, code, suffix) = unsafe { byte_vector.align_to::<u32>() };

        if !prefix.is_empty() || !suffix.is_empty() {
            error!("Shader bytecode is not properly aligned");
        }

        let shader_module_create_info = ShaderModuleCreateInfo::builder()
            .code_size(byte_vector.len())
            .code(code);

        unsafe {
            vkDevice::from(device)
                .create_shader_module(&shader_module_create_info, None)
                .unwrap()
        }
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            vkDevice::from(self.device.clone()).destroy_shader_module(self.module, None);
        }
    }
}

impl From<Shader> for vk::ShaderModule {
    fn from(value: Shader) -> Self {
        value.module
    }
}

impl From<&Shader> for vk::ShaderModule {
    fn from(value: &Shader) -> Self {
        value.module
    }
}
