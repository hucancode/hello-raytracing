use std::env;
use wgsl_toy::App;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    app.parse_args(args);
    event_loop.run_app(&mut app).unwrap();
}
