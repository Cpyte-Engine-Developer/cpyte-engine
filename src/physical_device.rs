use hashbrown::HashSet;
use itertools::Itertools;
use vulkanalia::vk::{self, InstanceV1_0, PhysicalDeviceType, StringArray, TRUE};

use crate::instance::Instance;

#[derive(Debug, Clone)]
pub(crate) struct PhysicalDevice {
    pub(crate) physical_device: vk::PhysicalDevice,
}

impl PhysicalDevice {
    pub(crate) fn new(instance: Instance, extensions: &[StringArray<256usize>]) -> PhysicalDevice {
        let physical_device = unsafe {
            *instance
                .instance
                .enumerate_physical_devices()
                .unwrap()
                .iter()
                .find(|physical_device| {
                    let physical_device_properties = instance
                        .instance
                        .get_physical_device_properties(**physical_device);

                    let physical_device_features = instance
                        .instance
                        .get_physical_device_features(**physical_device);

                    let physical_device_extensions = instance
                        .instance
                        .enumerate_device_extension_properties(**physical_device, None)
                        .unwrap()
                        .iter()
                        .map(|extension| extension.extension_name)
                        .unique()
                        .collect::<HashSet<_>>();

                    let suitable_device_type = (physical_device_properties.device_type
                        == PhysicalDeviceType::INTEGRATED_GPU
                        || physical_device_properties.device_type
                            == PhysicalDeviceType::DISCRETE_GPU)
                        && physical_device_features.sampler_anisotropy == TRUE;

                    let extensions = extensions.iter().copied().collect::<HashSet<_>>();

                    suitable_device_type && physical_device_extensions.is_superset(&extensions)
                })
                .unwrap()
        };

        Self { physical_device }
    }
}

impl From<PhysicalDevice> for vk::PhysicalDevice {
    fn from(value: PhysicalDevice) -> Self {
        value.physical_device
    }
}

impl From<&PhysicalDevice> for vk::PhysicalDevice {
    fn from(value: &PhysicalDevice) -> Self {
        value.physical_device
    }
}
