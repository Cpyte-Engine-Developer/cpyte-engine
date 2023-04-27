use itertools::Itertools;
use vulkanalia::{
    prelude::v1_0::Instance as vkInstance,
    vk::{
        make_version, ApplicationInfo, DebugUtilsMessengerCreateInfoEXT, HasBuilder,
        InstanceCreateInfo, InstanceV1_0, EXT_DEBUG_UTILS_EXTENSION,
    },
    window as vk_window,
};
use winit::window::Window;

use crate::entry::Entry;

#[derive(Debug, Clone)]
pub(crate) struct Instance {
    pub(crate) instance: vkInstance,
}

impl Instance {
    pub(crate) fn new(
        window: &Window,
        layers: &[*const i8],
        debug_messenger_create_info: &mut DebugUtilsMessengerCreateInfoEXT,
        entry: &Entry,
    ) -> Self {
        let app_info = ApplicationInfo::builder()
            .application_name(b"")
            .application_version(0)
            .engine_name(b"Cpyte engine")
            .engine_version(make_version(1, 0, 0))
            .api_version(make_version(1, 0, 0));

        let mut extensions = vk_window::get_required_instance_extensions(window)
            .iter()
            .map(|extension| extension.as_ptr())
            .collect_vec();

        #[cfg(debug_assertions)]
        extensions.push(EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());

        let instance_create_info = InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(layers)
            .push_next(debug_messenger_create_info);

        let instance = unsafe {
            entry
                .entry
                .create_instance(&instance_create_info, None)
                .unwrap()
        };

        Self { instance }
    }

    pub(crate) fn destroy(&self) {
        unsafe { self.instance.destroy_instance(None) };
    }
}

impl From<Instance> for vkInstance {
    fn from(value: Instance) -> Self {
        value.instance
    }
}

impl From<&Instance> for vkInstance {
    fn from(value: &Instance) -> Self {
        value.instance.clone()
    }
}
