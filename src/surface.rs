use vulkanalia::{
    vk::{KhrSurfaceExtension, SurfaceKHR},
    window::create_surface,
};
use winit::window::Window;

use crate::instance::Instance;

#[derive(Debug, Clone)]
pub(crate) struct Surface {
    pub(crate) surface: SurfaceKHR,
    instance: Instance,
}

impl Surface {
    pub(crate) fn new(instance: Instance, window: &Window) -> Self {
        let surface = unsafe { create_surface(&instance.instance, window).unwrap() };

        Self { surface, instance }
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            self.instance
                .instance
                .destroy_surface_khr(self.surface, None);
        }
    }
}

impl From<Surface> for SurfaceKHR {
    fn from(value: Surface) -> Self {
        value.surface
    }
}

impl From<&Surface> for SurfaceKHR {
    fn from(value: &Surface) -> Self {
        value.surface
    }
}
