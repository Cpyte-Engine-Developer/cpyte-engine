use std::rc::Rc;

use image::{io::Reader as ImageReader, RgbaImage};
use rand::{thread_rng, Rng};
use vulkanalia::vk::{
    Extent3D, Format, ImageAspectFlags, ImageTiling, ImageUsageFlags, MemoryPropertyFlags,
    SampleCountFlags,
};

use crate::{
    device::Device, image::Image, instance::Instance, physical_device::PhysicalDevice,
    sampler::Sampler,
};

#[derive(Default, Clone, Debug)]
pub(crate) struct Texture {
    pub(crate) id: usize,
    pub(crate) path: String,
    pub(crate) image: Rc<RgbaImage>,
}

impl Texture {
    pub(crate) fn new(image_path: &str) -> Self {
        let image = Rc::new(
            ImageReader::open(image_path)
                .unwrap()
                .decode()
                .unwrap()
                .into_rgba8(),
        );

        Self {
            id: thread_rng().gen::<usize>(),
            path: image_path.to_string(),
            image,
        }
    }

    pub(crate) fn create_image(
        extent: Extent3D,
        msaa_sample_count: SampleCountFlags,
        device: Device,
        instance: Instance,
        physical_device: PhysicalDevice,
    ) -> Image {
        Image::new(
            extent,
            msaa_sample_count,
            device,
            instance,
            physical_device,
            Image::mip_levels(extent),
            Format::R8G8B8A8_SRGB,
            ImageTiling::OPTIMAL,
            ImageUsageFlags::SAMPLED
                | ImageUsageFlags::TRANSFER_SRC
                | ImageUsageFlags::TRANSFER_DST,
            MemoryPropertyFlags::DEVICE_LOCAL,
            ImageAspectFlags::COLOR,
        )
    }

    pub(crate) fn create_sampler(device: Device, mip_levels: u32) -> Sampler {
        Sampler::new(device, mip_levels)
    }
}
