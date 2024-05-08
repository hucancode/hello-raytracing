mod bvh;
mod camera;
mod material;
mod sphere;
use std::f32::consts::PI;

pub use bvh::Node;
pub use bvh::Tree;
pub use bvh::Triangle;
pub use camera::Camera;
use glam::Vec3;
pub use material::Material;
pub use material::DIELECTRIC;
pub use material::METAL;
use rand::prelude::*;
pub use sphere::Sphere;

use crate::geometry::Mesh;

pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<Sphere>,
    pub tris_bvh: Tree,
}

impl Scene {
    pub fn new() -> Self {
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
        Self {
            camera,
            objects,
            tris_bvh: Default::default(),
        }
    }
    pub fn new_simple() -> Self {
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
        Self {
            camera,
            objects,
            tris_bvh: Default::default(),
        }
    }

    pub fn new_suzane() -> Self {
        let mesh = Mesh::load_obj(
            include_bytes!("../assets/suzanne.obj"),
            Material::new_lambertian(Vec3::new(0.5, 0.5, 0.6)),
        );
        let mut tree: Tree = mesh.into();
        let mesh = Mesh::load_obj(
            include_bytes!("../assets/cube2.obj"),
            Material::new_metal(Vec3::new(0.5, 0.5, 0.6), 0.2),
        );
        tree.add_mesh(mesh);
        tree.build();
        let camera = Camera::new(
            Vec3::new(0.0, 2.2, 4.5),
            Vec3::new(0.0, 0.0, -4.5),
            5.6,
            0.25,
            PI * 0.3,
        );
        Self {
            camera,
            objects: Vec::new(),
            tris_bvh: tree,
        }
    }
    pub fn new_cube() -> Self {
        let mesh = Mesh::load_obj(
            include_bytes!("../assets/cube2.obj"),
            Material::new_lambertian(Vec3::new(0.5, 0.5, 0.6)),
        );
        let mut tree: Tree = mesh.into();
        tree.build();
        let camera = Camera::new(
            Vec3::new(0.0, 2.2, 6.5),
            Vec3::new(0.0, 0.1, -3.0),
            2.2,
            0.0,
            PI * 0.3,
        );
        Self {
            camera,
            objects: Vec::new(),
            tris_bvh: tree,
        }
    }
    pub fn new_quad() -> Self {
        let mesh = Mesh::load_obj(
            include_bytes!("../assets/quad.obj"),
            Material::new_lambertian(Vec3::new(0.5, 0.5, 0.6)),
        );
        let mut tree: Tree = mesh.into();
        tree.build();
        let camera = Camera::new(
            Vec3::new(0.0, 0.2, 3.5),
            Vec3::new(0.0, 0.1, -3.0),
            2.2,
            0.0,
            PI * 0.3,
        );
        Self {
            camera,
            objects: Vec::new(),
            tris_bvh: tree,
        }
    }
}
