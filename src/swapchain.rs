use itertools::Itertools;
use vulkanalia::{
    vk::{
        self, ColorSpaceKHR, CompositeAlphaFlagsKHR, DeviceV1_0, Extent2D, Format, Handle,
        HasBuilder, ImageAspectFlags, ImageUsageFlags, ImageView, KhrSurfaceExtension,
        KhrSwapchainExtension, PresentModeKHR, SharingMode, SurfaceFormatKHR,
        SwapchainCreateInfoKHR, SwapchainKHR,
    },
    Device as vkDevice,
};
use winit::window::Window;

use crate::{
    device::Device, image::Image, instance::Instance, physical_device::PhysicalDevice,
    semaphore::Semaphore, surface::Surface,
};

#[derive(Clone, Debug)]
pub(crate) struct Swapchain {
    pub(crate) extent: Extent2D,
    pub(crate) swapchain: SwapchainKHR,
    pub(crate) images: Vec<vk::Image>,
    pub(crate) image_views: Vec<ImageView>,
    device: Device,
}

impl Swapchain {
    pub(crate) fn new(
        instance: Instance,
        physical_device: PhysicalDevice,
        surface: Surface,
        graphics_queue_family_index: u32,
        present_queue_family_index: u32,
        device: Device,
        window: &Window,
    ) -> Self {
        let format = Self::format(instance.clone(), physical_device.clone(), surface.clone());
        let extent = Self::extent(window);
        let old_swapchain = Self::old_swapchain();

        let swapchain = Self::create_swapchain(
            instance,
            physical_device,
            surface,
            format,
            extent,
            graphics_queue_family_index,
            present_queue_family_index,
            device.clone(),
            old_swapchain,
        );

        let images = Self::create_images(device.clone(), swapchain);
        let image_views =
            Self::create_image_views(images.as_slice(), format.format, device.clone());

        Self {
            extent,
            swapchain,
            images,
            image_views,
            device,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn create_swapchain(
        instance: Instance,
        physical_device: PhysicalDevice,
        surface: Surface,
        format: SurfaceFormatKHR,
        extent: Extent2D,
        graphics_queue_family_index: u32,
        present_queue_family_index: u32,
        device: Device,
        old_swapchain: SwapchainKHR,
    ) -> SwapchainKHR {
        let available_present_modes = unsafe {
            instance
                .instance
                .get_physical_device_surface_present_modes_khr(
                    physical_device.physical_device,
                    surface.surface,
                )
                .unwrap()
        };

        let present_mode = available_present_modes
            .iter()
            .cloned()
            .find(|present_mode| *present_mode == PresentModeKHR::MAILBOX)
            .unwrap_or(PresentModeKHR::FIFO);

        let surface_capabilities = unsafe {
            instance
                .instance
                .get_physical_device_surface_capabilities_khr(
                    physical_device.physical_device,
                    surface.surface,
                )
                .unwrap()
        };

        let (queue_family_indices, sharing_mode) =
            if graphics_queue_family_index != present_queue_family_index {
                (
                    vec![graphics_queue_family_index, present_queue_family_index],
                    SharingMode::CONCURRENT,
                )
            } else {
                (vec![], SharingMode::EXCLUSIVE)
            };

        let swapchain_create_info = SwapchainCreateInfoKHR::builder()
            .surface(surface.surface)
            .min_image_count(surface_capabilities.min_image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(sharing_mode)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(old_swapchain);

        unsafe {
            vkDevice::from(device)
                .create_swapchain_khr(&swapchain_create_info, None)
                .unwrap()
        }
    }

    pub(crate) fn format(
        instance: Instance,
        physical_device: PhysicalDevice,
        surface: Surface,
    ) -> SurfaceFormatKHR {
        let available_formats = unsafe {
            instance
                .instance
                .get_physical_device_surface_formats_khr(
                    physical_device.physical_device,
                    surface.surface,
                )
                .unwrap()
        };

        #[allow(clippy::or_fun_call)]
        available_formats
            .iter()
            .cloned()
            .find(|format| {
                format.format == Format::B8G8R8A8_SRGB
                    && format.color_space == ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or(
                SurfaceFormatKHR::builder()
                    .format(Format::B8G8R8A8_UNORM)
                    .color_space(ColorSpaceKHR::SRGB_NONLINEAR)
                    .build(),
            )
    }

    fn extent(window: &Window) -> Extent2D {
        Extent2D::builder()
            .width(window.inner_size().width)
            .height(window.inner_size().height)
            .build()
    }

    pub(crate) fn old_swapchain() -> SwapchainKHR {
        SwapchainKHR::null()
    }

    fn create_images(device: Device, swapchain: SwapchainKHR) -> Vec<vk::Image> {
        unsafe {
            vkDevice::from(device)
                .get_swapchain_images_khr(swapchain)
                .unwrap()
        }
    }

    fn create_image_views(images: &[vk::Image], format: Format, device: Device) -> Vec<ImageView> {
        images
            .iter()
            .map(|swapchain_image| {
                Image::create_view(
                    device.clone(),
                    *swapchain_image,
                    format,
                    ImageAspectFlags::COLOR,
                    1,
                )
            })
            .collect_vec()
    }

    pub(crate) fn next_image_index(&self, wait_semaphore: Semaphore) -> usize {
        let index = unsafe {
            vkDevice::from(self.device.clone())
                .acquire_next_image_khr(
                    self.swapchain,
                    u64::MAX,
                    wait_semaphore.semaphore,
                    vk::Fence::null(),
                )
                .unwrap()
                .0
        };
        index as usize
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            vkDevice::from(self.device.clone()).destroy_swapchain_khr(self.swapchain, None);

            let device = &self.device;

            self.image_views.iter().for_each(|image_view| {
                vkDevice::from(device).destroy_image_view(*image_view, None)
            });
        }
    }
}

impl From<Swapchain> for SwapchainKHR {
    fn from(value: Swapchain) -> Self {
        value.swapchain
    }
}

impl From<&Swapchain> for SwapchainKHR {
    fn from(value: &Swapchain) -> Self {
        value.swapchain
    }
}
