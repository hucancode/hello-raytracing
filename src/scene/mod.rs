pub mod bvh;
mod camera;
mod material;
mod render_ppm;
mod scene_sphere;
mod scene_tris;
mod sphere;
pub use camera::Camera;
pub use material::Material;
pub use scene_sphere::SceneSphere;
pub use scene_tris::SceneTris;

pub trait Scene {
    fn init(&mut self);
    fn draw(&mut self);
    fn set_time(&mut self, time: u32);
    fn resize(&mut self, width: u32, height: u32);
}

impl Scene for SceneTris {
    fn init(&mut self) {
        self.renderer.set_camera(&self.camera);
        self.write_tree_data();
    }
    fn draw(&mut self) {
        self.renderer.draw()
    }
    fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height)
    }
    fn set_time(&mut self, time: u32) {
        self.renderer.set_time(time)
    }
}

impl Scene for SceneSphere {
    fn init(&mut self) {
        self.renderer.set_camera(&self.camera);
        self.write_scene_data();
    }
    fn draw(&mut self) {
        self.renderer.draw()
    }
    fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height)
    }
    fn set_time(&mut self, time: u32) {
        self.renderer.set_time(time)
    }
}
