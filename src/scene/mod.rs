mod camera;
mod material;
mod sphere;
pub use camera::Camera;
use glam::Vec3;
pub use material::Material;
pub use sphere::Sphere;

pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<Sphere>,
}

impl Scene {
    pub fn new() -> Self {
        let yellow = Vec3::new(0.98, 0.89, 0.69);
        let red = Vec3::new(0.953, 0.545, 0.659);
        let base = Vec3::new(0.12, 0.12, 0.18);
        let blue = Vec3::new(0.54, 0.7, 0.98);
        let black = Vec3::new(0.06, 0.06, 0.1);
        Self {
            camera: Camera::new(Vec3::new(0.0, 0.2, 10.0), Vec3::new(0.0, 0.1, 9.0), 1.2),
            objects: vec![
                Sphere::new_lambertian(Vec3::new(0.0, -100.5, -1.0), 100.0, base),
                Sphere::new_dielectric(Vec3::new(-1.0, 0.0, -1.0), 0.5, 1.5),
                Sphere::new_lambertian(Vec3::new(0.0, 0.0, -1.0), 0.5, black),
                Sphere::new_metal(Vec3::new(1.0, 0., -1.0), 0.5, yellow, 0.1),
                Sphere::new_lambertian(Vec3::new(-0.7, -0.3, -0.2), 0.2, red),
                Sphere::new_metal(Vec3::new(-0.3, -0.4, -0.4), 0.1, blue, 0.9),
                Sphere::new_dielectric(Vec3::new(0.2, -0.38, -0.16), 0.12, 0.1),
            ],
        }
    }
}
