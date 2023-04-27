mod buffer;
mod command_buffer;
mod command_executor;
mod command_pool;
mod debug_messenger;
mod descriptor_pool;
mod descriptor_set;
mod device;
mod entity;
mod entry;
mod fence;
mod framebuffer;
mod image;
mod instance;
mod memory;
mod model;
mod physical_device;
mod pipeline;
mod queue;
mod queue_family_index;
mod render_pass;
mod renderer;
mod sampler;
mod scene_graph;
mod semaphore;
mod shader;
mod surface;
mod swapchain;
mod texture;
mod ubo;
mod validation_layers;
mod vertex;
mod window;

use nalgebra::{UnitQuaternion, Vector3};
use std::{cell::RefCell, rc::Rc, time::Instant};
use vulkanalia::vk::SampleCountFlags;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use crate::{entity::Entity, scene_graph::SceneGraph};
use renderer::Renderer;

fn main() {
    pretty_env_logger::init();

    let event_loop = EventLoop::new();

    let entity_1 = Entity::new(
        Vector3::default(),
        UnitQuaternion::from_axis_angle(&Vector3::z_axis(), 90.0f32.to_radians()),
        Vector3::new(1.0, 1.0, 1.0),
        "/home/arman/Документы/может быть нужное/cpyte-engine (копия)/data/3d-models/viking_room.obj",
        Some("/home/arman/Документы/может быть нужное/cpyte-engine (копия)/data/textures/viking_room.png"),
    );

    let entity_2 = Entity::new(
        Vector3::new(1.0, 0.0, 0.0),
        UnitQuaternion::from_axis_angle(&Vector3::z_axis(), 90.0f32.to_radians()),
        Vector3::new(1.0, 1.0, 1.0),
        "/home/arman/Документы/может быть нужное/cpyte-engine (копия)/data/3d-models/bochka.obj",
        Some(
            "/home/arman/Документы/может быть нужное/cpyte-engine (копия)/data/textures/bochka.png",
        ),
    );

    let scene_graph = Rc::new(RefCell::new(SceneGraph::new()));
    scene_graph.borrow_mut().insert("Entity", entity_1);
    scene_graph.borrow_mut().insert("Entity 1", entity_2);

    let mut renderer = Renderer::new(&event_loop, Rc::clone(&scene_graph), SampleCountFlags::_1);

    let start_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                let exec_time = start_time.elapsed().as_secs_f32();

                scene_graph.borrow_mut().on_update(exec_time);
                renderer.draw_frame(exec_time);
            }
            _ => {}
        }
    });
}
