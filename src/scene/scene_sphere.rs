use std::cmp::min;
use std::f32::consts::PI;
use std::mem::size_of;

use crate::renderer::RenderOutput;
use crate::renderer::Renderer;
pub use crate::scene::camera::Camera;
pub use crate::scene::material::DIELECTRIC;
pub use crate::scene::material::METAL;
pub use crate::scene::sphere::Sphere;
use glam::Vec3;
use rand::prelude::*;
use wgpu::BufferBindingType;

const MAX_OBJECT_IN_SCENE: u64 = 100;

pub struct SceneSphere {
    pub renderer: Renderer,
    pub camera: Camera,
    pub objects: Vec<Sphere>,
}

impl SceneSphere {
    pub fn write_scene_data(&mut self) {
        let data = bytemuck::cast_slice(self.objects.as_slice());
        let n = min(
            data.len(),
            MAX_OBJECT_IN_SCENE as usize * size_of::<Sphere>(),
        );
        self.renderer.write_buffer(&data[0..n], 0)
    }
    pub async fn new(output: RenderOutput) -> Self {
        let black = Vec3::new(0.06, 0.06, 0.1);
        let mut rng = rand::thread_rng();
        let mut objects = Vec::new();
        let base_radius = 1.0;
        let base_center = Vec3::ZERO;
        let camera_position = base_center + Vec3::new(0.0, 0.0, 3.5);
        let camera = Camera::new(camera_position, base_center, 3.5, 0.04, PI * 0.2);
        objects.push(Sphere::new_lambertian(base_center, base_radius, black));
        let mut generate = |x: f32, y: f32, z: f32| {
            if rng.gen_bool(0.6) {
                return;
            }
            let dir = Vec3::new(x, y, z);
            let mat = rng.gen_range(1..=3);
            let size = rng.gen_range(0.05..0.15) * base_radius;
            let pos = dir.normalize() * (base_radius + size) + base_center;
            let obj = match mat {
                METAL => {
                    let color = Vec3::new(rng.gen(), rng.gen(), rng.gen());
                    let fuzzy = rng.gen();
                    Sphere::new_metal(pos, size, color, fuzzy)
                }
                DIELECTRIC => {
                    let ir = rng.gen_range(0.1..0.4);
                    Sphere::new_dielectric(pos, size, ir)
                }
                _ => {
                    let color = Vec3::new(rng.gen(), rng.gen(), rng.gen());
                    Sphere::new_lambertian(pos, size, color)
                }
            };
            objects.push(obj);
        };
        for x in -2..2 {
            for y in -2..2 {
                for z in 0..4 {
                    generate(x as f32, y as f32, z as f32);
                }
            }
        }
        let renderer = Renderer::new(
            output,
            vec![
                (
                    BufferBindingType::Storage { read_only: true },
                    MAX_OBJECT_IN_SCENE * size_of::<Sphere>() as u64,
                ), // spheres
            ],
            include_str!("../shaders/shader_sphere.wgsl"),
        )
        .await;
        Self {
            renderer,
            camera,
            objects,
        }
    }
    pub async fn new_simple(output: RenderOutput) -> Self {
        let yellow = Vec3::new(0.98, 0.89, 0.69);
        let red = Vec3::new(0.953, 0.545, 0.659);
        let base = Vec3::new(0.12, 0.12, 0.18);
        let blue = Vec3::new(0.54, 0.7, 0.98);
        let black = Vec3::new(0.06, 0.06, 0.1);
        let camera = Camera::new(
            Vec3::new(0.0, 0.2, 1.5),
            Vec3::new(0.0, 0.1, -3.0),
            2.2,
            0.1,
            PI * 0.3,
        );
        let objects = vec![
            Sphere::new_lambertian(Vec3::new(0.0, -100.5, -1.0), 100.0, base),
            Sphere::new_dielectric(Vec3::new(-1.0, 0.0, -1.0), 0.5, 1.5),
            Sphere::new_lambertian(Vec3::new(0.0, 0.0, -1.0), 0.5, black),
            Sphere::new_metal(Vec3::new(1.0, 0., -1.0), 0.5, yellow, 0.1),
            Sphere::new_lambertian(Vec3::new(-0.7, -0.3, -0.2), 0.2, red),
            Sphere::new_metal(Vec3::new(-0.3, -0.4, -0.4), 0.1, blue, 0.9),
            Sphere::new_dielectric(Vec3::new(0.2, -0.38, -0.16), 0.12, 0.1),
        ];
        let renderer = Renderer::new(
            output,
            vec![
                (
                    BufferBindingType::Storage { read_only: true },
                    MAX_OBJECT_IN_SCENE * size_of::<Sphere>() as u64,
                ), // spheres
            ],
            include_str!("../shaders/shader_sphere.wgsl"),
        )
        .await;
        Self {
            renderer,
            camera,
            objects,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::render_ppm::render_ppm;
    use super::*;
    use crate::scene::Scene;
    use std::io::Write;

    #[test]
    fn globe() {
        let width = 1024;
        let height = 768;
        let mut scene = pollster::block_on(SceneSphere::new(RenderOutput::Headless(
            width, height,
        )));
        scene.init();
        let content = render_ppm(&mut scene.renderer);
        let mut file = std::fs::File::create("globe.ppm").unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn sphere() {
        let width = 1024;
        let height = 768;
        let mut scene = pollster::block_on(SceneSphere::new_simple(RenderOutput::Headless(
            width, height,
        )));
        scene.init();
        let content = render_ppm(&mut scene.renderer);
        let mut file = std::fs::File::create("sphere.ppm").unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }
}
