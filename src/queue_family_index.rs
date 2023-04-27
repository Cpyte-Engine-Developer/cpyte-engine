use vulkanalia::vk::{InstanceV1_0, KhrSurfaceExtension, QueueFlags};

use crate::{instance::Instance, physical_device::PhysicalDevice, surface::Surface};

pub(crate) struct QueueFamilyIndex;

impl QueueFamilyIndex {
    pub(crate) fn graphics(instance: Instance, physical_device: PhysicalDevice) -> u32 {
        let physical_device_properties = unsafe {
            instance
                .instance
                .get_physical_device_queue_family_properties(physical_device.physical_device)
        };

        physical_device_properties
            .iter()
            .position(|queue_family_properties| {
                queue_family_properties
                    .queue_flags
                    .contains(QueueFlags::GRAPHICS)
            })
            .unwrap() as u32
    }

    pub(crate) fn present(
        instance: Instance,
        physical_device: PhysicalDevice,
        surface: Surface,
    ) -> u32 {
        let physical_device_properties = unsafe {
            instance
                .instance
                .get_physical_device_queue_family_properties(physical_device.physical_device)
        };

        physical_device_properties
            .iter()
            .enumerate()
            .position(|(i, _)| unsafe {
                instance
                    .instance
                    .get_physical_device_surface_support_khr(
                        physical_device.physical_device,
                        i as u32,
                        surface.surface,
                    )
                    .unwrap()
            })
            .unwrap() as u32
    }
}
