use vulkanalia::{
    vk::{
        self, DescriptorPoolCreateInfo, DescriptorPoolSize, DescriptorType, DeviceV1_0, HasBuilder,
    },
    Device as vkDevice,
};

use crate::{device::Device, entity::Entity};

#[derive(Debug, Clone)]
pub(crate) struct DescriptorPool {
    descriptor_pool: vk::DescriptorPool,
    device: Device,
}

impl DescriptorPool {
    pub(crate) fn new(device: Device, entities: &[&Entity]) -> Self {
        let ubo_pool_size = DescriptorPoolSize::builder()
            .type_(DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(entities.len() as u32);
        let image_sampler_pool_size = DescriptorPoolSize::builder()
            .type_(DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(entities.len() as u32);

        let descriptor_pool_sizes = &[ubo_pool_size, image_sampler_pool_size];
        let descriptor_pool_create_info = DescriptorPoolCreateInfo::builder()
            .pool_sizes(descriptor_pool_sizes)
            .max_sets(entities.len() as u32);

        let descriptor_pool = unsafe {
            vkDevice::from(device.clone())
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .unwrap()
        };

        Self {
            descriptor_pool,
            device,
        }
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            vkDevice::from(self.device.clone()).destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}

impl From<DescriptorPool> for vk::DescriptorPool {
    fn from(value: DescriptorPool) -> Self {
        value.descriptor_pool
    }
}

impl From<&DescriptorPool> for vk::DescriptorPool {
    fn from(value: &DescriptorPool) -> Self {
        value.descriptor_pool
    }
}
