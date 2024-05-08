use crate::{renderer::Renderer, renderer_tris::RendererTris, scene::Scene};
use std::{sync::Arc, time::Instant};
use winit::window::Window;
pub struct App {
    renderer: RendererTris,
    scene: Scene,
}

impl App {
    pub async fn new(window: Arc<Window>) -> Self {
        let renderer = RendererTris::new(window).await;
        let scene = Scene::new_suzane();
        Self { renderer, scene }
    }
    pub fn init(&mut self) {
        let app_init_timestamp = Instant::now();
        self.renderer.set_scene(&self.scene);
        println!("app initialized in {:?}", app_init_timestamp.elapsed());
    }
    pub fn update(&mut self, time: u32) {
        self.renderer.set_time(time)
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height)
    }

    pub fn draw(&mut self) {
        self.renderer.draw()
    }
}
