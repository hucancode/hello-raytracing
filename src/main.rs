use std::sync::Arc;

use wgsl_toy::run;
use winit::event_loop::EventLoop;
use winit::window::Window;
fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = Window::new(&event_loop).unwrap();
    window.set_title("Hello WGSL");
    env_logger::init();
    pollster::block_on(run(event_loop, Arc::new(window)));
}
