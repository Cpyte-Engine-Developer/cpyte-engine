use std::{marker::PhantomData, mem::size_of, ptr::copy_nonoverlapping as copy_memory};

use nalgebra::{Matrix4, Vector3};
use vulkanalia::{
    prelude::v1_0::Device as vkDevice,
    vk::{
        self, BufferCopy, BufferCreateInfo, BufferUsageFlags, DeviceMemory, DeviceSize, DeviceV1_0,
        Extent2D, HasBuilder, MemoryAllocateInfo, MemoryMapFlags, MemoryPropertyFlags, SharingMode,
    },
};

use crate::{
    command_executor::CommandExecutor, command_pool::CommandPool, device::Device,
    instance::Instance, memory::Memory, physical_device::PhysicalDevice, queue::Queue, ubo::Ubo,
    vertex::Vertex,
};

pub(crate) type IndexBuffer = Buffer<u32>;
pub(crate) type VertexBuffer = Buffer<Vertex>;
pub(crate) type UniformBuffer = Buffer<Ubo>;

#[derive(Clone, Debug)]
pub(crate) struct Buffer<T: Clone> {
    buffer: vk::Buffer,
    pub(crate) memory: DeviceMemory,
    device: Device,
    instance: Instance,
    physical_device: PhysicalDevice,
    phantom: PhantomData<T>,
}

impl<T: Clone> Buffer<T> {
    pub(crate) fn new(
        size: DeviceSize,
        usage_flags: BufferUsageFlags,
        device: Device,
        instance: Instance,
        physical_device: PhysicalDevice,
        memory_property_flags: MemoryPropertyFlags,
    ) -> Self {
        let buffer = Self::create_self(size, usage_flags, device.clone());
        let memory = Self::create_memory(
            device.clone(),
            buffer,
            instance.clone(),
            physical_device.clone(),
            memory_property_flags,
        );

        Self {
            buffer,
            memory,
            device,
            phantom: PhantomData,
            instance,
            physical_device,
        }
    }

    pub(crate) fn from_indices(
        indices: &[u32],
        device: Device,
        instance: Instance,
        physical_device: PhysicalDevice,
        command_pool: CommandPool,
        graphics_queue: Queue,
    ) -> IndexBuffer {
        let buffer = Buffer::new(
            (size_of::<u32>() * indices.len()) as u64,
            BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::INDEX_BUFFER,
            device,
            instance,
            physical_device,
            MemoryPropertyFlags::DEVICE_LOCAL,
        );

        buffer.fill(indices, command_pool, graphics_queue);

        buffer
    }

    pub(crate) fn from_staging_data(
        values: &[T],
        device: Device,
        instance: Instance,
        physical_device: PhysicalDevice,
    ) -> Buffer<T> {
        Buffer::new(
            (size_of::<T>() * values.len()) as u64,
            BufferUsageFlags::TRANSFER_SRC,
            device,
            instance,
            physical_device,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
        )
    }

    pub(crate) fn from_vertices(
        vertices: &[Vertex],
        device: Device,
        instance: Instance,
        physical_device: PhysicalDevice,
        command_pool: CommandPool,
        graphics_queue: Queue,
    ) -> VertexBuffer {
        let buffer = Buffer::new(
            (size_of::<Vertex>() * vertices.len()) as u64,
            BufferUsageFlags::VERTEX_BUFFER | BufferUsageFlags::TRANSFER_DST,
            device,
            instance,
            physical_device,
            MemoryPropertyFlags::DEVICE_LOCAL,
        );

        buffer.fill(vertices, command_pool, graphics_queue);

        buffer
    }

    pub(crate) fn from_uniform_data(
        device: Device,
        instance: Instance,
        physical_device: PhysicalDevice,
    ) -> UniformBuffer {
        Buffer::new(
            size_of::<Ubo>() as u64,
            BufferUsageFlags::UNIFORM_BUFFER,
            device,
            instance,
            physical_device,
            MemoryPropertyFlags::HOST_COHERENT | MemoryPropertyFlags::HOST_VISIBLE,
        )
    }

    pub(crate) fn fill(&self, values: &[T], command_pool: CommandPool, graphics_queue: Queue) {
        let size = Self::size(values.len());
        let staging_buffer = Buffer::from_staging_data(
            values,
            self.device.clone(),
            self.instance.clone(),
            self.physical_device.clone(),
        );

        self.copy_memory(staging_buffer.clone(), size, values);
        self.copy_buffer(
            command_pool,
            graphics_queue,
            size,
            staging_buffer.clone(),
            self.clone(),
        );

        staging_buffer.destroy();
    }

    fn size(len: usize) -> u64 {
        (size_of::<T>() * len) as u64
    }

    pub(crate) fn copy_memory(&self, staging_buffer: Buffer<T>, buffer_size: u64, values: &[T]) {
        unsafe {
            let mapped_memory = vkDevice::from(self.device.clone())
                .map_memory(
                    staging_buffer.memory,
                    0,
                    buffer_size,
                    MemoryMapFlags::empty(),
                )
                .unwrap();

            copy_memory(values.as_ptr(), mapped_memory.cast(), values.len());

            vkDevice::from(self.device.clone()).unmap_memory(staging_buffer.memory);
        }
    }

    pub(crate) fn copy_buffer(
        &self,
        command_pool: CommandPool,
        graphics_queue: Queue,
        buffer_size: u64,
        src_buffer: Buffer<T>,
        dst_buffer: Buffer<T>,
    ) {
        CommandExecutor::execute(
            command_pool,
            self.device.clone(),
            |command_buffer| {
                let copy_buffer = BufferCopy::builder().size(buffer_size);

                unsafe {
                    vkDevice::from(self.device.clone()).cmd_copy_buffer(
                        command_buffer.into(),
                        src_buffer.buffer,
                        dst_buffer.buffer,
                        &[copy_buffer],
                    );
                }
            },
            graphics_queue,
        );
    }

    fn create_self(size: DeviceSize, usage_flags: BufferUsageFlags, device: Device) -> vk::Buffer {
        let buffer_create_info = BufferCreateInfo::builder()
            .size(size)
            .usage(usage_flags)
            .sharing_mode(SharingMode::EXCLUSIVE);

        unsafe {
            vkDevice::from(device)
                .create_buffer(&buffer_create_info, None)
                .unwrap()
        }
    }

    fn create_memory(
        device: Device,
        buffer: vk::Buffer,
        instance: Instance,
        physical_device: PhysicalDevice,
        memory_property_flags: MemoryPropertyFlags,
    ) -> DeviceMemory {
        let buffer_memory_requirements =
            unsafe { vkDevice::from(device.clone()).get_buffer_memory_requirements(buffer) };

        let memory_allocate_info = MemoryAllocateInfo::builder()
            .allocation_size(buffer_memory_requirements.size)
            .memory_type_index(Memory::type_index(
                instance,
                physical_device,
                memory_property_flags,
                buffer_memory_requirements,
            ));

        let buffer_memory = unsafe {
            vkDevice::from(device.clone())
                .allocate_memory(&memory_allocate_info, None)
                .unwrap()
        };

        unsafe {
            vkDevice::from(device)
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .unwrap()
        };

        buffer_memory
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            vkDevice::from(self.device.clone()).destroy_buffer(self.buffer, None);
            vkDevice::from(self.device.clone()).free_memory(self.memory, None);
        }
    }
}

impl UniformBuffer {
    pub(crate) fn update(&mut self, extent: Extent2D) {
        let view_matrix = Matrix4::look_at_rh(
            &Vector3::new(2.0, 2.0, 2.0).into(),
            &Vector3::new(0.0, 0.0, 0.0).into(),
            &Vector3::new(0.0, 0.0, 0.1),
        );

        let mut perspective_matrix = Matrix4::new_perspective(
            extent.width as f32 / extent.height as f32,
            45.0f32.to_radians(),
            0.1,
            10.0,
        );

        perspective_matrix[(1, 1)] *= -1.0;

        let ubo = Ubo::new(view_matrix, perspective_matrix);

        self.copy_memory(self.clone(), size_of::<Ubo>() as u64, &[ubo]);
    }

    // pub(crate) fn create_perspective_matrix(
    //     aspect: f32,
    //     fov_y: f32,
    //     far: f32,
    //     near: f32,
    // ) -> Matrix4<f32> {
    //     let tan_half_fov_y = (fov_y / 2.0).tan();

    //     let mut matrix = Matrix4::zeros();

    //     matrix[(0, 0)] = 1.0 / (aspect * tan_half_fov_y);
    //     matrix[(1, 1)] = 1.0 / tan_half_fov_y;
    //     matrix[(2, 2)] = far / (near - far);
    //     matrix[(2, 3)] = -(far * near) / (far - near);
    //     matrix[(3, 2)] = -1.0;

    //     matrix[(1, 1)] *= -1.0;

    //     matrix
    // }
}

impl<T: Clone> From<Buffer<T>> for vk::Buffer {
    fn from(value: Buffer<T>) -> Self {
        value.buffer
    }
}

impl<T: Clone> From<&Buffer<T>> for vk::Buffer {
    fn from(value: &Buffer<T>) -> Self {
        value.buffer
    }
}
