mod compute;
mod core;
mod gpu;
mod graphics;
mod physics;
mod testing;
mod utils;
mod app;

use winit::event_loop::{ControlFlow, EventLoop};
use crate::app::app::App;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
