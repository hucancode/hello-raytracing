mod camera;
mod material;
mod sphere;
pub use camera::Camera;
pub use material::Material;
pub use material::DIELECTRIC;
pub use material::METAL;
pub use sphere::Sphere;
use glam::Vec3;
use rand::prelude::*;

pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<Sphere>,
}

impl Scene {
    pub fn new() -> Self {
        let black = Vec3::new(0.06, 0.06, 0.1);
        let mut rng = rand::thread_rng();
        let mut objects = Vec::new();
        let base_radius = 1.4;
        let base_center = Vec3::new(0.0, -1.0, 0.0);
        objects.push(Sphere::new_lambertian(base_center, base_radius, black));
        for _ in 0..10 {
            let dir = Vec3::new(
                rng.gen::<f32>() - 0.5,
                rng.gen::<f32>() * 0.5,
                rng.gen::<f32>() * 0.5,
            );
            let mat = (rng.gen::<f32>() * 3.0 + 1.0) as u32;
            let size = (rng.gen::<f32>() * 0.2 * base_radius) + base_radius * 0.05;
            let pos = dir.normalize() * (base_radius + size) + base_center;
            let obj = match mat {
                METAL => {
                    let color = Vec3::new(rng.gen(), rng.gen(), rng.gen());
                    let fuzzy = rng.gen();
                    Sphere::new_metal(pos, size, color, fuzzy)
                }
                DIELECTRIC => {
                    let ir = rng.gen::<f32>()*0.3 + 0.1;
                    Sphere::new_dielectric(pos, size, ir)
                }
                _ => {
                    let color = Vec3::new(rng.gen(), rng.gen(), rng.gen());
                    Sphere::new_lambertian(pos, size, color)
                }
            };
            objects.push(obj);
        }
        let camera = Camera::new(Vec3::new(0.0, 2.0, 10.0), Vec3::ZERO, 1.2);
        Self { camera, objects }
    }
    pub fn _new_simple() -> Self {
        let yellow = Vec3::new(0.98, 0.89, 0.69);
        let red = Vec3::new(0.953, 0.545, 0.659);
        let base = Vec3::new(0.12, 0.12, 0.18);
        let blue = Vec3::new(0.54, 0.7, 0.98);
        let black = Vec3::new(0.06, 0.06, 0.1);
        let camera = Camera::new(Vec3::new(0.0, 0.2, 10.0), Vec3::new(0.0, 0.1, 9.0), 1.2);
        let objects = vec![
            Sphere::new_lambertian(Vec3::new(0.0, -100.5, -1.0), 100.0, base),
            Sphere::new_dielectric(Vec3::new(-1.0, 0.0, -1.0), 0.5, 1.5),
            Sphere::new_lambertian(Vec3::new(0.0, 0.0, -1.0), 0.5, black),
            Sphere::new_metal(Vec3::new(1.0, 0., -1.0), 0.5, yellow, 0.1),
            Sphere::new_lambertian(Vec3::new(-0.7, -0.3, -0.2), 0.2, red),
            Sphere::new_metal(Vec3::new(-0.3, -0.4, -0.4), 0.1, blue, 0.9),
            Sphere::new_dielectric(Vec3::new(0.2, -0.38, -0.16), 0.12, 0.1),
        ];
        Self { camera, objects }
    }
}
