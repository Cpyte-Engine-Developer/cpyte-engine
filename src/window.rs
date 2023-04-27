use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{self, WindowBuilder},
};

pub(crate) struct Window;

impl Window {
    #[allow(clippy::new_ret_no_self)]
    pub(crate) fn new(event_loop: &EventLoop<()>) -> window::Window {
        WindowBuilder::new()
            .with_title("Cpyte engine")
            .with_inner_size(LogicalSize {
                width: 798,
                height: 598,
            })
            .with_resizable(false)
            .build(event_loop)
            .unwrap()
    }
}
