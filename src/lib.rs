mod app;
mod geometry;
mod renderer;
mod scene;
use crate::app::App;
use std::{sync::Arc, time::Instant};
use winit::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub async fn run(event_loop: EventLoop<()>, window: Arc<Window>, args: Vec<String>) {
    let mut app = App::new(window.clone(), args).await;
    // let mut last_update_timestamp = Instant::now();
    let app_start_timestamp = Instant::now();
    let _ = event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);
        match event {
            Event::NewEvents(start_cause) => match start_cause {
                StartCause::Init => app.init(),
                StartCause::Poll => {
                    let time = app_start_timestamp.elapsed().as_millis();
                    app.update(time as u32);
                    window.request_redraw();
                }
                _ => {}
            },
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    app.resize(size.width, size.height);
                    window.request_redraw();
                }
                WindowEvent::RedrawRequested => app.draw(),
                WindowEvent::CloseRequested => elwt.exit(),
                _ => {}
            },
            _ => {}
        }
    });
}
