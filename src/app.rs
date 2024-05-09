use crate::renderer::RenderOutput;
use crate::scene::{Scene, SceneSphere, SceneTris};
use rand::Rng;
use std::{i8, sync::Arc, time::Instant};
use winit::window::Window;
pub struct App {
    scene: Box<dyn Scene>,
}

impl App {
    pub async fn new(window: Arc<Window>, args: Vec<String>) -> Self {
        let mut rng = rand::thread_rng();
        let j = rng.gen_range(1..=5);
        let i = args.get(1).map_or(j, |s| s.parse::<i8>().unwrap_or(j));
        let scene: Box<dyn Scene> = match i {
            1 => Box::new(SceneSphere::new_simple(RenderOutput::Window(window)).await),
            2 => Box::new(SceneSphere::new(RenderOutput::Window(window)).await),
            3 => Box::new(SceneTris::new_quad(RenderOutput::Window(window)).await),
            4 => Box::new(SceneTris::new_cube(RenderOutput::Window(window)).await),
            _ => Box::new(SceneTris::new_suzane(RenderOutput::Window(window)).await),
        };
        Self { scene }
    }
    pub fn init(&mut self) {
        let app_init_timestamp = Instant::now();
        self.scene.init();
        println!("app initialized in {:?}", app_init_timestamp.elapsed());
    }
    pub fn update(&mut self, time: u32) {
        self.scene.set_time(time)
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        self.scene.resize(width, height)
    }

    pub fn draw(&mut self) {
        self.scene.draw()
    }
}
