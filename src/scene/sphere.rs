use crate::scene::Material;
use bytemuck::{Pod, Zeroable};
use glam::Vec3;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Sphere {
    center: Vec3,
    radius: f32,
    material: Material,
}

impl Sphere {
    pub fn new_lambertian(center: Vec3, radius: f32, color: Vec3) -> Self {
        let material = Material::new_lambertian(color);
        Self {
            center,
            radius,
            material,
        }
    }
    pub fn new_metal(center: Vec3, radius: f32, color: Vec3, fuzzy: f32) -> Self {
        let material = Material::new_metal(color, fuzzy);
        Self {
            center,
            radius,
            material,
        }
    }
    pub fn new_dielectric(center: Vec3, radius: f32, ir: f32) -> Self {
        let material = Material::new_dielectric(ir);
        Self {
            center,
            radius,
            material,
        }
    }
}
