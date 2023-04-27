use vulkanalia::vk::{InstanceV1_0, MemoryPropertyFlags, MemoryRequirements};

use crate::{instance::Instance, physical_device::PhysicalDevice};

pub(crate) struct Memory;

impl Memory {
    pub(crate) fn type_index(
        instance: Instance,
        physical_device: PhysicalDevice,
        memory_property_flags: MemoryPropertyFlags,
        memory_requirements: MemoryRequirements,
    ) -> u32 {
        let memory_properties = unsafe {
            instance
                .instance
                .get_physical_device_memory_properties(physical_device.physical_device)
        };

        (0..memory_properties.memory_type_count)
            .find(|memory_type_index| {
                memory_properties.memory_types[*memory_type_index as usize]
                    .property_flags
                    .contains(memory_property_flags)
                    && (memory_requirements.memory_type_bits & (1 << memory_type_index)) != 0
            })
            .unwrap()
    }
}
