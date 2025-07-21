pub mod bvh;
mod camera;
mod material;
mod scene_sphere;
mod scene_tris;
mod sphere;
pub use camera::Camera;
pub use material::Material;
pub use scene_sphere::SceneSphere;
pub use scene_tris::SceneTris;
use crate::camera_controller::CameraUniform;

pub trait Scene {
    fn init(&mut self);
    fn draw(&mut self);
    fn set_time(&mut self, time: u32);
    fn resize(&mut self, width: u32, height: u32);
    fn update_camera(&mut self, camera: CameraUniform);
    fn reset_frame_count(&mut self);
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
    fn update_camera(&mut self, camera: CameraUniform) {
        self.renderer.update_camera_uniform(camera)
    }
    fn reset_frame_count(&mut self) {
        self.renderer.reset_frame_count()
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
    fn update_camera(&mut self, camera: CameraUniform) {
        self.renderer.update_camera_uniform(camera)
    }
    fn reset_frame_count(&mut self) {
        self.renderer.reset_frame_count()
    }
}

#[cfg(test)]
mod render_ppm;
#[cfg(test)]
pub use render_ppm::render_ppm;
