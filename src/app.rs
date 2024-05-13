use crate::renderer::RenderOutput;
use crate::scene::{Scene, SceneSphere, SceneTris};
use rand::Rng;
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use std::sync::Arc;
use std::{i8, time::Instant};
use winit::window::{Window, WindowId};

pub struct App {
    scene: Option<Box<dyn Scene>>,
    window: Option<Arc<Window>>,
    scene_id: i8,
    start_time_stamp: Instant,
}

impl Default for App {
    fn default() -> Self {
        Self {
            start_time_stamp: Instant::now(),
            scene_id: 0,
            scene: None,
            window: None,
        }
    }
}

impl App {
    pub fn parse_args(&mut self, args: Vec<String>) {
        let mut rng = rand::thread_rng();
        let j = rng.gen_range(1..=7);
        let i = args.get(1).map_or(j, |s| s.parse::<i8>().unwrap_or(j));
        self.scene_id = i;
    }
    async fn build_scene(&mut self) {
        if let Some(window) = self.window.as_ref() {
            let mut scene: Box<dyn Scene> = match self.scene_id {
                2 => Box::new(SceneSphere::new(RenderOutput::Window(window.clone())).await),
                3 => Box::new(SceneTris::new_quad(RenderOutput::Window(window.clone())).await),
                4 => Box::new(SceneTris::new_cube(RenderOutput::Window(window.clone())).await),
                5 => Box::new(SceneTris::new_suzane(RenderOutput::Window(window.clone())).await),
                6 => Box::new(SceneTris::new_lucy(RenderOutput::Window(window.clone())).await),
                7 => Box::new(SceneTris::new_dragon(RenderOutput::Window(window.clone())).await),
                _ => Box::new(SceneSphere::new_simple(RenderOutput::Window(window.clone())).await),
            };
            scene.init();
            self.scene = Some(scene);
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Window::default_attributes()).unwrap());
        self.window = Some(window);
        pollster::block_on(self.build_scene());
    }
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Poll {
            let time = self.start_time_stamp.elapsed().as_millis() as u32;
            if let Some(scene) = self.scene.as_mut() {
                scene.set_time(time);
            }
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => if let Some(scene) = self.scene.as_mut() {
                scene.draw();
            },
            WindowEvent::Resized(size) => if let Some(scene) = self.scene.as_mut() {
                scene.resize(size.width, size.height);
            },
            _ => {}
        }
    }
}