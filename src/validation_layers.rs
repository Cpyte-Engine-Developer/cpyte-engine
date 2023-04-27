use hashbrown::HashSet;
use log::error;
use vulkanalia::vk::{EntryV1_0, ExtensionName};

use crate::entry::Entry;

pub(crate) struct ValidationLayers;

impl ValidationLayers {
    #[allow(clippy::new_ret_no_self)]
    pub(crate) fn new(entry: &Entry) -> Vec<*const i8> {
        let mut layers = Vec::new();

        #[cfg(debug_assertions)]
        {
            const VALIDATION_LAYER: ExtensionName =
                ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

            let available_layers = unsafe {
                entry
                    .entry
                    .enumerate_instance_layer_properties()
                    .unwrap()
                    .iter()
                    .map(|layer| layer.layer_name)
                    .collect::<HashSet<_>>()
            };

            if !available_layers.contains(&VALIDATION_LAYER) {
                error!("{} dont work", VALIDATION_LAYER);
            }

            layers.push(VALIDATION_LAYER.as_ptr());
        }

        layers
    }
}
