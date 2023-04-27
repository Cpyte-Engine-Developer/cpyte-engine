use std::mem::size_of;

use vulkanalia::{
    vk::{
        self, Buffer, CopyDescriptorSet, DescriptorBufferInfo, DescriptorImageInfo, DescriptorPool,
        DescriptorSetAllocateInfo, DescriptorSetLayout, DescriptorSetLayoutBinding,
        DescriptorSetLayoutCreateInfo, DescriptorType, DeviceV1_0, HasBuilder, ImageLayout,
        ImageView, ShaderStageFlags, WriteDescriptorSet,
    },
    Device as vkDevice,
};

use crate::{device::Device, sampler::Sampler, ubo::Ubo};

#[derive(Clone, Debug)]
pub(crate) struct DescriptorSet {
    descriptor_set: vk::DescriptorSet,
    pub(crate) layout: DescriptorSetLayout,
    device: Device,
}

impl DescriptorSet {
    pub(crate) fn new(
        device: Device,
        descriptor_pool: DescriptorPool,
        uniform_buffer: Buffer,
        texture_image_view: ImageView,
        texture_sampler: Sampler,
    ) -> Self {
        let layout = Self::create_layout(device.clone());
        let descriptor_set = Self::create_descriptor_set(
            layout,
            descriptor_pool,
            device.clone(),
            uniform_buffer,
            texture_image_view,
            texture_sampler,
        );

        Self {
            descriptor_set,
            layout,
            device,
        }
    }

    fn create_descriptor_set(
        descriptor_set_layout: DescriptorSetLayout,
        descriptor_pool: DescriptorPool,
        device: Device,
        uniform_buffer: Buffer,
        texture_image_view: ImageView,
        texture_sampler: Sampler,
    ) -> vk::DescriptorSet {
        let descriptor_set_layouts = vec![descriptor_set_layout; 1];
        let descriptor_set_allocate_info = DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&descriptor_set_layouts);

        let descriptor_set = unsafe {
            vkDevice::from(device.clone())
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .unwrap()
        }[0];

        let descriptor_buffer_info = DescriptorBufferInfo::builder()
            .buffer(uniform_buffer)
            .offset(0)
            .range(size_of::<Ubo>() as u64);

        let descriptor_image_info = DescriptorImageInfo::builder()
            .image_layout(ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(texture_image_view)
            .sampler(texture_sampler.sampler);

        let descriptor_buffer_infos = &[descriptor_buffer_info];
        let descriptor_image_infos = &[descriptor_image_info];

        let ubo_write_descriptor_set = WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(DescriptorType::UNIFORM_BUFFER)
            .buffer_info(descriptor_buffer_infos);
        let image_write_descriptor_set = WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(descriptor_image_infos);

        unsafe {
            vkDevice::from(device).update_descriptor_sets(
                &[ubo_write_descriptor_set, image_write_descriptor_set],
                &[] as &[CopyDescriptorSet],
            );
        }

        descriptor_set
    }

    fn create_layout(device: Device) -> vk::DescriptorSetLayout {
        let ubo_layout_binding = DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(ShaderStageFlags::VERTEX);
        let image_sampler_binding = DescriptorSetLayoutBinding::builder()
            .binding(1)
            .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(ShaderStageFlags::FRAGMENT);

        let descriptor_set_layout_bindings = &[ubo_layout_binding, image_sampler_binding];
        let descriptor_set_layout_create_info =
            DescriptorSetLayoutCreateInfo::builder().bindings(descriptor_set_layout_bindings);

        unsafe {
            vkDevice::from(device)
                .create_descriptor_set_layout(&descriptor_set_layout_create_info, None)
                .unwrap()
        }
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            vkDevice::from(self.device.clone()).destroy_descriptor_set_layout(self.layout, None);
        }
    }
}

impl From<DescriptorSet> for vk::DescriptorSet {
    fn from(value: DescriptorSet) -> Self {
        value.descriptor_set
    }
}

impl From<&DescriptorSet> for vk::DescriptorSet {
    fn from(value: &DescriptorSet) -> Self {
        value.descriptor_set
    }
}
