use hashbrown::HashSet;
use itertools::Itertools;
use vulkanalia::{
    prelude::v1_0::Device as vkDevice,
    vk::{
        DeviceCreateInfo, DeviceQueueCreateInfo, DeviceV1_0, HasBuilder, PhysicalDeviceFeatures,
        SampleCountFlags, StringArray,
    },
};

use crate::{
    instance::Instance, physical_device::PhysicalDevice, queue_family_index::QueueFamilyIndex,
    surface::Surface,
};

#[derive(Clone, Debug)]
pub(crate) struct Device {
    device: vkDevice,
}

impl Device {
    pub(crate) fn new(
        instance: Instance,
        physical_device: PhysicalDevice,
        surface: Surface,
        extensions: &[StringArray<256usize>],
        layers: &[*const i8],
        msaa_sample_count: SampleCountFlags,
    ) -> Self {
        let graphics_queue_family_index =
            QueueFamilyIndex::graphics(instance.clone(), physical_device.clone());
        let present_queue_family_index =
            QueueFamilyIndex::present(instance.clone(), physical_device.clone(), surface);

        let queue_priorities = &[1.0];
        let unique_queue_family_indices =
            HashSet::<u32>::from([graphics_queue_family_index, present_queue_family_index]);
        let unique_queue_families_create_info = unique_queue_family_indices
            .iter()
            .map(|queue_family_index| {
                DeviceQueueCreateInfo::builder()
                    .queue_family_index(*queue_family_index)
                    .queue_priorities(queue_priorities)
            })
            .collect_vec();
        let physical_device_features = PhysicalDeviceFeatures::builder()
            .sampler_anisotropy(true)
            .sample_rate_shading(msaa_sample_count != SampleCountFlags::_1);

        let extensions = extensions
            .iter()
            .map(|extension| extension.as_ptr())
            .collect::<Vec<_>>();

        let device_create_info = DeviceCreateInfo::builder()
            .queue_create_infos(&unique_queue_families_create_info)
            .enabled_layer_names(layers)
            .enabled_features(&physical_device_features)
            .enabled_extension_names(&extensions);

        let device = unsafe {
            instance
                .instance
                .create_device(physical_device.physical_device, &device_create_info, None)
                .unwrap()
        };

        Self { device }
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}

impl From<Device> for vkDevice {
    fn from(value: Device) -> Self {
        value.device
    }
}

impl From<&Device> for vkDevice {
    fn from(value: &Device) -> Self {
        value.device.clone()
    }
}
