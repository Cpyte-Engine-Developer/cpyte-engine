use std::{cell::RefCell, rc::Rc};

use hashbrown::HashMap;
use itertools::Itertools;
use vulkanalia::{
    vk::{
        self, DeviceV1_0, Extent3D, HasBuilder, InstanceV1_0, PipelineStageFlags, PresentInfoKHR,
        SampleCountFlags, SubmitInfo, SwapchainKHR, KHR_SWAPCHAIN_EXTENSION,
    },
    Device as vkDevice,
};
use winit::event_loop::EventLoop;

use crate::{
    buffer::Buffer, command_buffer::CommandBuffer, command_pool::CommandPool,
    debug_messenger::DebugMessenger, descriptor_pool::DescriptorPool,
    descriptor_set::DescriptorSet, device::Device, entry::Entry, fence::Fence,
    framebuffer::Framebuffer, image::Image, instance::Instance, physical_device::PhysicalDevice,
    pipeline::Pipeline, queue::Queue, queue_family_index::QueueFamilyIndex,
    render_pass::RenderPass, sampler::Sampler, scene_graph::SceneGraph, semaphore::Semaphore,
    surface::Surface, swapchain::Swapchain, texture::Texture, ubo::Ubo,
    validation_layers::ValidationLayers, vertex::Vertex, window::Window,
};

pub(crate) struct Renderer {
    entry: Entry,
    instance: Instance,
    window: winit::window::Window,
    surface: Surface,
    #[cfg(debug_assertions)]
    debug_messenger: DebugMessenger,
    pub(crate) device: Device,
    graphics_queue: Queue,
    present_queue: Queue,
    old_swapchain: SwapchainKHR,
    swapchain: Swapchain,
    render_pass: RenderPass,
    pipeline: Pipeline,
    framebuffers: Vec<Framebuffer>,
    command_pool: CommandPool,
    color_image: Option<Image>,
    command_buffers: Vec<CommandBuffer>,
    wait_semaphores: Vec<Semaphore>,
    signal_semaphores: Vec<Semaphore>,
    unsignaled_fences: Vec<Fence>,
    signaled_fences: Vec<Fence>,
    scene_graph: Rc<RefCell<SceneGraph>>,
    uniform_buffers: Vec<Buffer<Ubo>>,
    texture_images: HashMap<usize, Image>,
    texture_samplers: HashMap<usize, Sampler>,
    descriptor_pool: DescriptorPool,
    vertex_buffers: HashMap<usize, Buffer<Vertex>>,
    index_buffers: HashMap<usize, Buffer<u32>>,
    descriptor_sets: HashMap<usize, DescriptorSet>,
    depth_image: Image,
    frame: usize,
}

const MAX_FLIGHT_FRAMES_COUNT: usize = 2;

impl Renderer {
    pub(crate) fn new(
        event_loop: &EventLoop<()>,
        scene_graph: Rc<RefCell<SceneGraph>>,
        msaa_sample_count: SampleCountFlags,
    ) -> Self {
        let entry = Entry::new();

        let window = Window::new(event_loop);

        let validation_layers = ValidationLayers::new(&entry);

        let mut debug_messenger_create_info = DebugMessenger::create_info();

        let instance = Instance::new(
            &window,
            validation_layers.as_slice(),
            &mut debug_messenger_create_info,
            &entry,
        );

        #[cfg(debug_assertions)]
        let debug_messenger =
            DebugMessenger::new(instance.clone(), &mut debug_messenger_create_info);

        let surface = Surface::new(instance.clone(), &window);

        let extensions = [KHR_SWAPCHAIN_EXTENSION.name];
        let msaa_sample_count = SampleCountFlags::_1;

        let physical_device = PhysicalDevice::new(instance.clone(), &extensions);
        let device = Device::new(
            instance.clone(),
            physical_device.clone(),
            surface.clone(),
            &extensions,
            validation_layers.as_slice(),
            msaa_sample_count,
        );

        let graphics_queue_family_index =
            QueueFamilyIndex::graphics(instance.clone(), physical_device.clone());
        let graphics_queue = Queue::new(device.clone(), graphics_queue_family_index);

        let present_queue_family_index =
            QueueFamilyIndex::present(instance.clone(), physical_device.clone(), surface.clone());
        let present_queue = Queue::new(device.clone(), present_queue_family_index);

        let old_swapchain = Swapchain::old_swapchain();
        let swapchain = Swapchain::new(
            instance.clone(),
            physical_device.clone(),
            surface.clone(),
            graphics_queue_family_index,
            present_queue_family_index,
            device.clone(),
            &window,
        );

        let render_pass = RenderPass::new(
            device.clone(),
            Swapchain::format(instance.clone(), physical_device.clone(), surface.clone()).format,
            Image::depth_format(instance.clone(), physical_device.clone()),
            Swapchain::format(instance.clone(), physical_device.clone(), surface.clone()).format,
            msaa_sample_count,
        );

        let command_pool = CommandPool::new(
            device.clone(),
            QueueFamilyIndex::graphics(instance.clone(), physical_device.clone()),
        );

        let uniform_buffers = swapchain
            .images
            .iter()
            .map(|_| {
                Buffer::<Ubo>::from_uniform_data(
                    device.clone(),
                    instance.clone(),
                    physical_device.clone(),
                )
            })
            .collect_vec();

        let descriptor_pool = DescriptorPool::new(
            device.clone(),
            scene_graph
                .borrow()
                .entities_with_names
                .values()
                .collect_vec()
                .as_slice(),
        );

        let texture_images = scene_graph
            .borrow()
            .entities_with_names
            .values()
            .map(|entity| {
                let id = entity.id;

                let extent = Extent3D::builder()
                    .width(entity.model.texture.image.width())
                    .height(entity.model.texture.image.height())
                    .depth(1)
                    .build();

                let image = Texture::create_image(
                    extent,
                    msaa_sample_count,
                    device.clone(),
                    instance.clone(),
                    physical_device.clone(),
                );

                image.fill(
                    Rc::clone(&entity.model.texture.image),
                    command_pool.clone(),
                    graphics_queue.clone(),
                );

                (id, image)
            })
            .collect::<HashMap<usize, Image>>();

        let texture_samplers = scene_graph
            .borrow()
            .entities_with_names
            .values()
            .zip(texture_images.values())
            .map(|(entity, image)| {
                let id = entity.id;

                let sampler = Texture::create_sampler(device.clone(), image.mip_levels);

                (id, sampler)
            })
            .collect::<HashMap<usize, Sampler>>();

        let vertex_buffers = scene_graph
            .borrow()
            .entities_with_names
            .values()
            .map(|entity| {
                let id = entity.id;

                let vertex_buffer = Buffer::<Vertex>::from_vertices(
                    entity.model.vertices.as_slice(),
                    device.clone(),
                    instance.clone(),
                    physical_device.clone(),
                    command_pool.clone(),
                    graphics_queue.clone(),
                );

                (id, vertex_buffer)
            })
            .collect::<HashMap<usize, Buffer<Vertex>>>();

        let index_buffers = scene_graph
            .borrow()
            .entities_with_names
            .values()
            .map(|entity| {
                let id = entity.id;

                let index_buffer = Buffer::<u32>::from_indices(
                    entity.model.indices.as_slice(),
                    device.clone(),
                    instance.clone(),
                    physical_device.clone(),
                    command_pool.clone(),
                    graphics_queue.clone(),
                );

                (id, index_buffer)
            })
            .collect::<HashMap<usize, Buffer<u32>>>();

        let descriptor_sets = uniform_buffers
            .iter()
            .zip(scene_graph.borrow().entities_with_names.values())
            .map(|(uniform_buffer, entity)| {
                let id = entity.id;

                let descriptor_set = DescriptorSet::new(
                    device.clone(),
                    descriptor_pool.clone().into(),
                    uniform_buffer.into(),
                    texture_images[&id].view,
                    texture_samplers[&id].clone(),
                );

                (id, descriptor_set)
            })
            .collect::<HashMap<usize, DescriptorSet>>();

        let pipeline = Pipeline::new(
            device.clone(),
            descriptor_sets.values().next().unwrap().layout,
            render_pass.clone(),
            msaa_sample_count,
        );

        let extent = Extent3D::builder()
            .width(swapchain.extent.width)
            .height(swapchain.extent.height)
            .depth(1)
            .build();

        let color_image = if msaa_sample_count > SampleCountFlags::_1 {
            Some(Image::new_unresolved(
                extent,
                msaa_sample_count,
                device.clone(),
                instance.clone(),
                physical_device.clone(),
                Swapchain::format(instance.clone(), physical_device.clone(), surface.clone())
                    .format,
            ))
        } else {
            None
        };

        let depth_image = Image::new_depth(
            extent,
            instance.clone(),
            physical_device,
            msaa_sample_count,
            device.clone(),
        );

        let framebuffers = swapchain
            .image_views
            .iter()
            .map(|image_view| {
                Framebuffer::new(
                    device.clone(),
                    image_view,
                    render_pass.clone(),
                    swapchain.extent,
                    depth_image.view,
                    color_image.as_ref().map(|image| image.view),
                )
            })
            .collect_vec();

        let command_buffers = framebuffers
            .iter()
            .map(|_| CommandBuffer::new(device.clone(), command_pool.clone()))
            .collect_vec();

        scene_graph
            .borrow_mut()
            .entities_with_names
            .iter_mut()
            .for_each(|(_, entity)| {
                entity.model.texture = Texture::new(&entity.model.texture.path);
            });

        let wait_semaphores = (0..MAX_FLIGHT_FRAMES_COUNT)
            .map(|_| Semaphore::new(device.clone()))
            .collect_vec();
        let signal_semaphores = (0..MAX_FLIGHT_FRAMES_COUNT)
            .map(|_| Semaphore::new(device.clone()))
            .collect_vec();

        let signaled_fences = (0..MAX_FLIGHT_FRAMES_COUNT)
            .map(|_| Fence::new(device.clone(), true))
            .collect_vec();
        let unsignaled_fences = (0..MAX_FLIGHT_FRAMES_COUNT)
            .map(|_| Fence::new(device.clone(), false))
            .collect_vec();

        let frame = 0;

        Self {
            entry,
            instance,
            #[cfg(debug_assertions)]
            debug_messenger,
            window,
            surface,
            device,
            graphics_queue,
            present_queue,
            old_swapchain,
            swapchain,
            render_pass,
            pipeline,
            framebuffers,
            command_pool,
            color_image,
            command_buffers,
            wait_semaphores,
            signal_semaphores,
            unsignaled_fences,
            signaled_fences,
            scene_graph,
            uniform_buffers,
            texture_images,
            texture_samplers,
            descriptor_pool,
            vertex_buffers,
            index_buffers,
            descriptor_sets,
            depth_image,
            frame,
        }
    }

    fn check_msaa_sample_count(
        instance: Instance,
        physical_device: vk::PhysicalDevice,
        msaa_sample_count: SampleCountFlags,
    ) -> bool {
        let physical_device_properties = unsafe {
            instance
                .instance
                .get_physical_device_properties(physical_device)
        };

        let available_sample_counts = physical_device_properties
            .limits
            .framebuffer_color_sample_counts
            & physical_device_properties
                .limits
                .framebuffer_depth_sample_counts;

        let all_sample_counts = [
            SampleCountFlags::_64,
            SampleCountFlags::_32,
            SampleCountFlags::_16,
            SampleCountFlags::_8,
            SampleCountFlags::_4,
            SampleCountFlags::_2,
            SampleCountFlags::_1,
        ];

        all_sample_counts
            .iter()
            .cloned()
            .find(|sample_count| available_sample_counts.contains(*sample_count))
            .map_or(false, |_| true)
    }

    pub(crate) fn draw_frame(&mut self, exec_time: f32) {
        self.signaled_fences[self.frame].wait();

        let image_index = self
            .swapchain
            .next_image_index(self.wait_semaphores[self.frame].clone());

        if !self.unsignaled_fences[self.frame].is_null() {
            self.unsignaled_fences[self.frame].wait();
        }

        self.command_buffers[image_index].reset();

        self.uniform_buffers[image_index].update(self.swapchain.extent);

        self.command_buffers[image_index].start_recording(
            self.swapchain.extent,
            self.render_pass.clone(),
            self.framebuffers[image_index].clone(),
            self.pipeline.clone(),
        );

        self.scene_graph
            .borrow()
            .entities_with_names
            .values()
            .for_each(|entity| {
                self.command_buffers[image_index].record_drawing(
                    self.vertex_buffers[&entity.id].clone(),
                    self.index_buffers[&entity.id].clone(),
                    self.pipeline.layout,
                    self.descriptor_sets[&entity.id].clone().into(),
                    entity.transform_matrix().as_slice(),
                    entity.model.indices.as_slice(),
                );
            });

        self.command_buffers[image_index].finish_recording();

        let wait_semaphores = &[self.wait_semaphores[self.frame].semaphore];
        let wait_stages = &[PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.command_buffers[image_index].clone().into()];
        let signal_semaphores = &[self.signal_semaphores[self.frame].semaphore];

        let submit_info = SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores)
            .build();

        self.signaled_fences[self.frame].reset();
        self.graphics_queue
            .submit(submit_info, self.signaled_fences[self.frame].clone());

        let swapchains = &[self.swapchain.swapchain];
        let image_indices = &[image_index as u32];

        let present_info = PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices)
            .build();

        self.present_queue.present(present_info);

        self.frame = (self.frame + 1) % MAX_FLIGHT_FRAMES_COUNT;
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            vkDevice::from(self.device.clone())
                .device_wait_idle()
                .unwrap();

            self.texture_samplers.values().for_each(Sampler::destroy);
            self.texture_images.values().for_each(Image::destroy);

            self.depth_image.destroy();

            self.descriptor_sets
                .values()
                .for_each(DescriptorSet::destroy);

            self.index_buffers.values().for_each(Buffer::destroy);
            self.vertex_buffers.values().for_each(Buffer::destroy);

            self.descriptor_pool.destroy();
            self.uniform_buffers.iter().for_each(Buffer::destroy);

            self.unsignaled_fences.iter().for_each(Fence::destroy);
            self.signaled_fences.iter().for_each(Fence::destroy);

            self.signal_semaphores.iter().for_each(Semaphore::destroy);
            self.wait_semaphores.iter().for_each(Semaphore::destroy);

            if let Some(color_image) = self.color_image.clone() {
                color_image.destroy();
            }

            self.command_pool.destroy();
            self.framebuffers.iter().for_each(Framebuffer::destroy);

            self.pipeline.destroy();
            self.render_pass.destroy();
            self.swapchain.destroy();
            self.device.destroy();
            self.surface.destroy();
            #[cfg(debug_assertions)]
            self.debug_messenger.destroy();
            self.instance.destroy();
        }
    }
}
