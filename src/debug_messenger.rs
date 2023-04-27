use std::ffi::{c_void, CStr};

use log::*;
use vulkanalia::vk::{
    Bool32, DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT,
    DebugUtilsMessengerCallbackDataEXT, DebugUtilsMessengerCreateInfoEXT, DebugUtilsMessengerEXT,
    ExtDebugUtilsExtension, HasBuilder, FALSE,
};

use crate::instance::Instance;

#[derive(Debug, Clone)]
pub(crate) struct DebugMessenger {
    debug_messenger: DebugUtilsMessengerEXT,
    instance: Instance,
}

impl DebugMessenger {
    pub(crate) fn new(
        instance: Instance,
        create_info: &mut DebugUtilsMessengerCreateInfoEXT,
    ) -> Self {
        let debug_messenger = unsafe {
            instance
                .instance
                .create_debug_utils_messenger_ext(create_info, None)
                .unwrap()
        };

        Self {
            debug_messenger,
            instance,
        }
    }

    pub(crate) fn create_info() -> DebugUtilsMessengerCreateInfoEXT {
        DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .user_callback(Some(Self::debug_callback))
            .build()
    }

    #[cfg(debug_assertions)]
    extern "system" fn debug_callback(
        severity: DebugUtilsMessageSeverityFlagsEXT,
        type_: DebugUtilsMessageTypeFlagsEXT,
        data: *const DebugUtilsMessengerCallbackDataEXT,
        _: *mut c_void,
    ) -> Bool32 {
        let data = unsafe { *data };
        let message = unsafe { CStr::from_ptr(data.message) }.to_string_lossy();

        match severity {
            DebugUtilsMessageSeverityFlagsEXT::ERROR => error!("[{:?}] {}", type_, message),
            DebugUtilsMessageSeverityFlagsEXT::WARNING => warn!("[{:?}] {}", type_, message),
            DebugUtilsMessageSeverityFlagsEXT::INFO => info!("[{:?}] {}", type_, message),
            DebugUtilsMessageSeverityFlagsEXT::VERBOSE => trace!("[{:?}] {}", type_, message),
            _ => trace!("[{:?}] {}", type_, message),
        }

        FALSE
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            self.instance
                .instance
                .destroy_debug_utils_messenger_ext(self.debug_messenger, None);
        }
    }
}

impl From<DebugMessenger> for DebugUtilsMessengerEXT {
    fn from(value: DebugMessenger) -> Self {
        value.debug_messenger
    }
}

impl From<&DebugMessenger> for DebugUtilsMessengerEXT {
    fn from(value: &DebugMessenger) -> Self {
        value.debug_messenger
    }
}
