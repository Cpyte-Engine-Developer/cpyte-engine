use std::{ptr::copy_nonoverlapping as copy_memory, rc::Rc};

use image::RgbaImage;
use log::error;
use vulkanalia::{
    vk::{
        self, AccessFlags, BufferImageCopy, BufferMemoryBarrier, DependencyFlags, DeviceMemory,
        DeviceV1_0, Extent3D, Filter, Format, FormatFeatureFlags, HasBuilder, ImageAspectFlags,
        ImageBlit, ImageCreateInfo, ImageLayout, ImageMemoryBarrier, ImageSubresourceLayers,
        ImageSubresourceRange, ImageTiling, ImageType, ImageUsageFlags, ImageView,
        ImageViewCreateInfo, ImageViewType, InstanceV1_0, MemoryAllocateInfo, MemoryBarrier,
        MemoryMapFlags, MemoryPropertyFlags, Offset3D, PipelineStageFlags, SampleCountFlags,
        SharingMode, QUEUE_FAMILY_IGNORED,
    },
    Device as vkDevice,
};

use crate::{
    buffer::Buffer, command_executor::CommandExecutor, command_pool::CommandPool, device::Device,
    instance::Instance, memory::Memory, physical_device::PhysicalDevice, queue::Queue,
};

#[derive(Clone, Debug)]
pub(crate) struct Image {
    pub(crate) vk_image: vk::Image,
    pub(crate) memory: DeviceMemory,
    pub(crate) view: ImageView,
    pub(crate) mip_levels: u32,
    pub(crate) extent: Extent3D,
    device: Device,
    instance: Instance,
    physical_device: PhysicalDevice,
}

impl Image {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        extent: Extent3D,
        msaa_sample_count: SampleCountFlags,
        device: Device,
        instance: Instance,
        physical_device: PhysicalDevice,
        mip_levels: u32,
        format: Format,
        image_tiling: ImageTiling,
        image_usage_flags: ImageUsageFlags,
        memory_property_flags: MemoryPropertyFlags,
        image_aspect_flags: ImageAspectFlags,
    ) -> Self {
        let vk_image = Self::create_image(
            extent,
            mip_levels,
            format,
            image_tiling,
            image_usage_flags,
            msaa_sample_count,
            device.clone(),
        );

        let memory = Self::create_memory(
            device.clone(),
            vk_image,
            instance.clone(),
            physical_device.clone(),
            memory_property_flags,
        );

        Self::bind_memory(device.clone(), vk_image, memory);

        let view = Self::create_view(
            device.clone(),
            vk_image,
            format,
            image_aspect_flags,
            mip_levels,
        );

        Self {
            vk_image,
            memory,
            view,
            mip_levels,
            device,
            extent,
            instance,
            physical_device,
        }
    }

    pub(crate) fn new_unresolved(
        extent: Extent3D,
        msaa_sample_count: SampleCountFlags,
        device: Device,
        instance: Instance,
        physical_device: PhysicalDevice,
        swapchain_format: Format,
    ) -> Image {
        Image::new(
            extent,
            msaa_sample_count,
            device,
            instance,
            physical_device,
            1,
            swapchain_format,
            ImageTiling::OPTIMAL,
            ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::TRANSIENT_ATTACHMENT,
            MemoryPropertyFlags::DEVICE_LOCAL,
            ImageAspectFlags::COLOR,
        )
    }

    pub(crate) fn new_depth(
        extent: Extent3D,
        instance: Instance,
        physical_device: PhysicalDevice,
        msaa_sample_count: SampleCountFlags,
        device: Device,
    ) -> Self {
        Self::new(
            extent,
            msaa_sample_count,
            device,
            instance.clone(),
            physical_device.clone(),
            1,
            Self::depth_format(instance, physical_device),
            ImageTiling::OPTIMAL,
            ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            MemoryPropertyFlags::DEVICE_LOCAL,
            ImageAspectFlags::DEPTH,
        )
    }

    pub(crate) fn depth_format(instance: Instance, physical_device: PhysicalDevice) -> Format {
        let formats = &[Format::D32_SFLOAT, Format::D32_SFLOAT_S8_UINT];

        Image::supported_format(
            instance,
            physical_device,
            formats,
            ImageTiling::OPTIMAL,
            FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )
    }

    pub(crate) fn fill(
        &self,
        image: Rc<RgbaImage>,
        command_pool: CommandPool,
        graphics_queue: Queue,
    ) {
        let pixels = image.as_raw();

        let staging_buffer = Buffer::from_staging_data(
            pixels.as_slice(),
            self.device.clone(),
            self.instance.clone(),
            self.physical_device.clone(),
        );

        unsafe {
            let mapped_memory = vkDevice::from(self.device.clone())
                .map_memory(
                    staging_buffer.memory,
                    0,
                    self.size(),
                    MemoryMapFlags::empty(),
                )
                .unwrap();
            copy_memory(pixels.as_ptr(), mapped_memory.cast(), pixels.len());
            vkDevice::from(self.device.clone()).unmap_memory(staging_buffer.memory);
        }

        self.optimize(command_pool, graphics_queue, staging_buffer.clone());

        staging_buffer.destroy();
    }

    #[allow(clippy::too_many_arguments)]
    fn create_image(
        extent: Extent3D,
        mip_levels: u32,
        format: Format,
        tiling: ImageTiling,
        usage_flags: ImageUsageFlags,
        msaa_sample_count: SampleCountFlags,
        device: Device,
    ) -> vk::Image {
        let image_create_info = ImageCreateInfo::builder()
            .image_type(ImageType::_2D)
            .extent(extent)
            .mip_levels(mip_levels)
            .array_layers(1)
            .format(format)
            .tiling(tiling)
            .initial_layout(ImageLayout::UNDEFINED)
            .usage(usage_flags)
            .sharing_mode(SharingMode::EXCLUSIVE)
            .samples(msaa_sample_count);

        unsafe {
            vkDevice::from(device)
                .create_image(&image_create_info, None)
                .unwrap()
        }
    }

    pub(crate) fn mip_levels(extent: Extent3D) -> u32 {
        (extent.width.max(extent.height) as f32).log2().floor() as u32 + 1
    }

    #[allow(clippy::too_many_arguments)]
    fn optimize(
        &self,
        command_pool: CommandPool,
        graphics_queue: Queue,
        staging_buffer: Buffer<u8>,
    ) {
        self.transition_image_layout(
            command_pool.clone(),
            graphics_queue.clone(),
            ImageLayout::UNDEFINED,
            ImageLayout::TRANSFER_DST_OPTIMAL,
        );

        self.copy_buffer_to_image(
            command_pool.clone(),
            graphics_queue.clone(),
            staging_buffer,
            self.clone(),
        );

        Self::create_mipmaps(
            self.device.clone(),
            self.instance.clone(),
            self.physical_device.clone(),
            command_pool,
            graphics_queue,
            self.vk_image,
            self.mip_levels,
            self.extent,
            Format::R8G8B8A8_SRGB,
        );
    }

    fn transition_image_layout(
        &self,
        command_pool: CommandPool,
        graphics_queue: Queue,
        old_layout: ImageLayout,
        new_layout: ImageLayout,
    ) {
        let (src_access_mask, dst_access_mask, src_stage_mask, dst_stage_mask) =
            match (old_layout, new_layout) {
                (ImageLayout::UNDEFINED, ImageLayout::TRANSFER_DST_OPTIMAL) => (
                    AccessFlags::empty(),
                    AccessFlags::TRANSFER_WRITE,
                    PipelineStageFlags::TOP_OF_PIPE,
                    PipelineStageFlags::TRANSFER,
                ),
                (ImageLayout::TRANSFER_DST_OPTIMAL, ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
                    AccessFlags::TRANSFER_WRITE,
                    AccessFlags::SHADER_READ,
                    PipelineStageFlags::TRANSFER,
                    PipelineStageFlags::FRAGMENT_SHADER,
                ),
                _ => {
                    error!("Matches of image layout not founded");
                    (
                        AccessFlags::empty(),
                        AccessFlags::empty(),
                        PipelineStageFlags::empty(),
                        PipelineStageFlags::empty(),
                    )
                }
            };

        CommandExecutor::execute(
            command_pool,
            self.device.clone(),
            |command_buffer| {
                let subresource_range = ImageSubresourceRange::builder()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(self.mip_levels)
                    .base_array_layer(0)
                    .layer_count(1);

                let image_memory_barrier = ImageMemoryBarrier::builder()
                    .old_layout(old_layout)
                    .new_layout(new_layout)
                    .src_queue_family_index(QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(QUEUE_FAMILY_IGNORED)
                    .image(self.vk_image)
                    .subresource_range(subresource_range)
                    .src_access_mask(src_access_mask)
                    .dst_access_mask(dst_access_mask);

                unsafe {
                    vkDevice::from(self.device.clone()).cmd_pipeline_barrier(
                        command_buffer.into(),
                        src_stage_mask,
                        dst_stage_mask,
                        DependencyFlags::empty(),
                        &[] as &[MemoryBarrier],
                        &[] as &[BufferMemoryBarrier],
                        &[image_memory_barrier],
                    );
                };
            },
            graphics_queue,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create_mipmaps(
        device: Device,
        instance: Instance,
        physical_device: PhysicalDevice,
        command_pool: CommandPool,
        graphics_queue: Queue,
        vk_image: vk::Image,
        mip_levels: u32,
        extent: Extent3D,
        format: Format,
    ) {
        if unsafe {
            !instance
                .instance
                .get_physical_device_format_properties(physical_device.physical_device, format)
                .optimal_tiling_features
                .contains(FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR)
        } {
            error!(
                "Physical device dont support optimal tiling with linear filtering for {:?}",
                format
            );
        }

        CommandExecutor::execute(
            command_pool,
            device.clone(),
            |command_buffer| {
                let image_subresource_range = ImageSubresourceRange::builder()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .base_array_layer(0)
                    .layer_count(1)
                    .level_count(1);

                let mut image_memory_barrier = ImageMemoryBarrier::builder()
                    .image(vk_image)
                    .src_queue_family_index(QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(QUEUE_FAMILY_IGNORED)
                    .subresource_range(image_subresource_range);

                let mut mipmap_width = extent.width;
                let mut mipmap_height = extent.height;

                (1..mip_levels).for_each(|mip_level| {
                    image_memory_barrier.subresource_range.base_mip_level = mip_level - 1;
                    image_memory_barrier.old_layout = ImageLayout::TRANSFER_DST_OPTIMAL;
                    image_memory_barrier.new_layout = ImageLayout::TRANSFER_SRC_OPTIMAL;
                    image_memory_barrier.src_access_mask = AccessFlags::TRANSFER_WRITE;
                    image_memory_barrier.dst_access_mask = AccessFlags::TRANSFER_READ;

                    unsafe {
                        vkDevice::from(device.clone()).cmd_pipeline_barrier(
                            command_buffer.clone().into(),
                            PipelineStageFlags::TRANSFER,
                            PipelineStageFlags::TRANSFER,
                            DependencyFlags::empty(),
                            &[] as &[MemoryBarrier],
                            &[] as &[BufferMemoryBarrier],
                            &[image_memory_barrier],
                        );
                    };

                    let src_subresource_layers = ImageSubresourceLayers::builder()
                        .aspect_mask(ImageAspectFlags::COLOR)
                        .mip_level(mip_level - 1)
                        .base_array_layer(0)
                        .layer_count(1);

                    let dst_subresource_layers = ImageSubresourceLayers::builder()
                        .aspect_mask(ImageAspectFlags::COLOR)
                        .mip_level(mip_level)
                        .base_array_layer(0)
                        .layer_count(1);

                    let image_blit = ImageBlit::builder()
                        .src_offsets([
                            Offset3D::builder().x(0).y(0).z(0).build(),
                            Offset3D::builder()
                                .x(mipmap_width as i32)
                                .y(mipmap_height as i32)
                                .z(1)
                                .build(),
                        ])
                        .src_subresource(src_subresource_layers)
                        .dst_offsets([
                            Offset3D::builder().x(0).y(0).z(0).build(),
                            Offset3D::builder()
                                .x(if mipmap_width > 1 {
                                    mipmap_width / 2
                                } else {
                                    1
                                } as i32)
                                .y(if mipmap_height > 1 {
                                    mipmap_height / 2
                                } else {
                                    1
                                } as i32)
                                .z(1)
                                .build(),
                        ])
                        .dst_subresource(dst_subresource_layers);

                    unsafe {
                        vkDevice::from(device.clone()).cmd_blit_image(
                            command_buffer.clone().into(),
                            vk_image,
                            ImageLayout::TRANSFER_SRC_OPTIMAL,
                            vk_image,
                            ImageLayout::TRANSFER_DST_OPTIMAL,
                            &[image_blit],
                            Filter::LINEAR,
                        );
                    };

                    image_memory_barrier.old_layout = ImageLayout::TRANSFER_SRC_OPTIMAL;
                    image_memory_barrier.new_layout = ImageLayout::SHADER_READ_ONLY_OPTIMAL;
                    image_memory_barrier.src_access_mask = AccessFlags::TRANSFER_READ;
                    image_memory_barrier.dst_access_mask = AccessFlags::SHADER_READ;

                    unsafe {
                        vkDevice::from(device.clone()).cmd_pipeline_barrier(
                            command_buffer.clone().into(),
                            PipelineStageFlags::TRANSFER,
                            PipelineStageFlags::FRAGMENT_SHADER,
                            DependencyFlags::empty(),
                            &[] as &[MemoryBarrier],
                            &[] as &[BufferMemoryBarrier],
                            &[image_memory_barrier],
                        );
                    };

                    if mipmap_width > 1 {
                        mipmap_width /= 2;
                    }
                    if mipmap_height > 1 {
                        mipmap_height /= 2;
                    }
                });

                image_memory_barrier.subresource_range.base_mip_level = mip_levels - 1;
                image_memory_barrier.old_layout = ImageLayout::TRANSFER_DST_OPTIMAL;
                image_memory_barrier.new_layout = ImageLayout::SHADER_READ_ONLY_OPTIMAL;
                image_memory_barrier.src_access_mask = AccessFlags::TRANSFER_WRITE;
                image_memory_barrier.dst_access_mask = AccessFlags::SHADER_READ;

                unsafe {
                    vkDevice::from(device).cmd_pipeline_barrier(
                        command_buffer.into(),
                        PipelineStageFlags::TRANSFER,
                        PipelineStageFlags::FRAGMENT_SHADER,
                        DependencyFlags::empty(),
                        &[] as &[MemoryBarrier],
                        &[] as &[BufferMemoryBarrier],
                        &[image_memory_barrier],
                    );
                };
            },
            graphics_queue,
        );
    }

    fn copy_buffer_to_image(
        &self,
        command_pool: CommandPool,
        graphics_queue: Queue,
        src_buffer: Buffer<u8>,
        dst_image: Image,
    ) {
        CommandExecutor::execute(
            command_pool,
            self.device.clone(),
            |command_buffer| {
                let subresource_layers = ImageSubresourceLayers::builder()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .mip_level(0)
                    .base_array_layer(0)
                    .layer_count(1);

                let buffer_image_copy = BufferImageCopy::builder()
                    .buffer_offset(0)
                    .buffer_row_length(0)
                    .buffer_image_height(0)
                    .image_subresource(subresource_layers)
                    .image_offset(Offset3D::builder().x(0).y(0).z(0))
                    .image_extent(self.extent);

                unsafe {
                    vkDevice::from(self.device.clone()).cmd_copy_buffer_to_image(
                        command_buffer.into(),
                        src_buffer.into(),
                        dst_image.vk_image,
                        ImageLayout::TRANSFER_DST_OPTIMAL,
                        &[buffer_image_copy],
                    )
                };
            },
            graphics_queue,
        );
    }

    fn create_memory(
        device: Device,
        image: vk::Image,
        instance: Instance,
        physical_device: PhysicalDevice,
        memory_property_flags: MemoryPropertyFlags,
    ) -> DeviceMemory {
        let image_memory_requirements =
            unsafe { vkDevice::from(device.clone()).get_image_memory_requirements(image) };
        let image_memory_alloc_info = MemoryAllocateInfo::builder()
            .allocation_size(image_memory_requirements.size)
            .memory_type_index(Memory::type_index(
                instance,
                physical_device,
                memory_property_flags,
                image_memory_requirements,
            ));

        unsafe {
            vkDevice::from(device)
                .allocate_memory(&image_memory_alloc_info, None)
                .unwrap()
        }
    }

    pub(crate) fn create_view(
        device: Device,
        vk_image: vk::Image,
        format: Format,
        aspect_flags: ImageAspectFlags,
        mip_levels: u32,
    ) -> vk::ImageView {
        let image_subresource_range = ImageSubresourceRange::builder()
            .aspect_mask(aspect_flags)
            .base_mip_level(0)
            .level_count(mip_levels)
            .base_array_layer(0)
            .layer_count(1);

        let image_view_create_info = ImageViewCreateInfo::builder()
            .image(vk_image)
            .view_type(ImageViewType::_2D)
            .format(format)
            .subresource_range(image_subresource_range);

        unsafe {
            vkDevice::from(device)
                .create_image_view(&image_view_create_info, None)
                .unwrap()
        }
    }

    pub(crate) fn size(&self) -> u64 {
        (self.extent.width * self.extent.height * 4) as u64
    }

    pub(crate) fn supported_format(
        instance: Instance,
        physical_device: PhysicalDevice,
        formats: &[Format],
        image_tiling: ImageTiling,
        format_features: FormatFeatureFlags,
    ) -> Format {
        formats
            .iter()
            .cloned()
            .find(|format| {
                let format_properties = unsafe {
                    instance.instance.get_physical_device_format_properties(
                        physical_device.physical_device,
                        *format,
                    )
                };

                match image_tiling {
                    ImageTiling::OPTIMAL => format_properties
                        .optimal_tiling_features
                        .contains(format_features),
                    ImageTiling::LINEAR => format_properties
                        .linear_tiling_features
                        .contains(format_features),
                    _ => {
                        error!("Not known tiling arrangement");
                        false
                    }
                }
            })
            .unwrap()
    }

    fn bind_memory(device: Device, image: vk::Image, memory: DeviceMemory) {
        unsafe {
            vkDevice::from(device)
                .bind_image_memory(image, memory, 0)
                .unwrap()
        }
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            vkDevice::from(self.device.clone()).destroy_image(self.vk_image, None);
            vkDevice::from(self.device.clone()).free_memory(self.memory, None);
            vkDevice::from(self.device.clone()).destroy_image_view(self.view, None);
        }
    }
}

impl From<Image> for vk::Image {
    fn from(value: Image) -> Self {
        value.vk_image
    }
}

impl From<&Image> for vk::Image {
    fn from(value: &Image) -> Self {
        value.vk_image
    }
}
