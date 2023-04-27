use vulkanalia::{
    vk::{
        self, BorderColor, CompareOp, DeviceV1_0, Filter, HasBuilder, SamplerAddressMode,
        SamplerCreateInfo, SamplerMipmapMode,
    },
    Device as vkDevice,
};

use crate::device::Device;

#[derive(Debug, Clone)]
pub(crate) struct Sampler {
    pub(crate) sampler: vk::Sampler,
    device: Device,
}

impl Sampler {
    pub(crate) fn new(device: Device, mip_levels: u32) -> Self {
        let sampler_create_info = SamplerCreateInfo::builder()
            .mag_filter(Filter::LINEAR)
            .min_filter(Filter::LINEAR)
            .address_mode_u(SamplerAddressMode::REPEAT)
            .address_mode_v(SamplerAddressMode::REPEAT)
            .address_mode_w(SamplerAddressMode::REPEAT)
            .anisotropy_enable(true)
            .max_anisotropy(16.0)
            .border_color(BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(CompareOp::ALWAYS)
            .mipmap_mode(SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(mip_levels as f32);

        let sampler = unsafe {
            vkDevice::from(device.clone())
                .create_sampler(&sampler_create_info, None)
                .unwrap()
        };

        Self { sampler, device }
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            vkDevice::from(self.device.clone()).destroy_sampler(self.sampler, None);
        }
    }
}

impl From<Sampler> for vk::Sampler {
    fn from(value: Sampler) -> Self {
        value.sampler
    }
}

impl From<&Sampler> for vk::Sampler {
    fn from(value: &Sampler) -> Self {
        value.sampler
    }
}
