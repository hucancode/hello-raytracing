use crate::camera_controller::OrbitCamera;
use crate::renderer::RenderOutput;
use crate::scene::{Scene, SceneSphere, SceneTris};
use rand::Rng;
use std::sync::Arc;
use std::{i8, time::Instant};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

pub struct App {
    scene: Option<Box<dyn Scene>>,
    window: Option<Arc<Window>>,
    scene_id: i8,
    start_time_stamp: Instant,
    camera: Option<OrbitCamera>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            start_time_stamp: Instant::now(),
            scene_id: 0,
            scene: None,
            window: None,
            camera: None,
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
        let Some(window) = self.window.as_ref() else {
            return;
        };
        let render_output = RenderOutput::Window(window.clone());
        let mut scene: Box<dyn Scene> = match self.scene_id {
            2 => Box::new(SceneSphere::new(render_output).await),
            3 => Box::new(SceneTris::new_quad(render_output).await),
            4 => Box::new(SceneTris::new_cube(render_output).await),
            5 => Box::new(SceneTris::new_suzane(render_output).await),
            6 => Box::new(SceneTris::new_lucy(render_output).await),
            7 => Box::new(SceneTris::new_dragon(render_output).await),
            _ => Box::new(SceneSphere::new_simple(render_output).await),
        };
        scene.init();
        self.scene = Some(scene);
        let size = window.inner_size();
        let aspect_ratio = size.width as f32 / size.height as f32;
        self.camera = Some(OrbitCamera::new(aspect_ratio));
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
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
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if event == WindowEvent::CloseRequested {
            event_loop.exit();
        } else if let Some(scene) = self.scene.as_mut() {
            match event {
                WindowEvent::RedrawRequested => {
                    if let Some(camera) = self.camera.as_mut() {
                        if camera.has_moved {
                            scene.reset_frame_count();
                            camera.reset_movement_flag();
                        }
                        scene.update_camera(camera.to_uniform());
                    }
                    scene.draw();
                }
                WindowEvent::Resized(size) => {
                    scene.resize(size.width, size.height);
                    if let Some(camera) = self.camera.as_mut() {
                        camera.resize(size.width, size.height);
                    }
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    if let Some(camera) = self.camera.as_mut() {
                        camera.handle_mouse_input(state, button);
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    if let Some(camera) = self.camera.as_mut() {
                        camera.handle_mouse_motion(position);
                    }
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    if let Some(camera) = self.camera.as_mut() {
                        camera.handle_scroll(delta);
                    }
                }
                _ => {}
            }
        }
    }
}
