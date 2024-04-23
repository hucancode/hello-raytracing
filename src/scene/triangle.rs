use crate::scene::Material;
use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Triangle {
    a: Vec4,
    b: Vec4,
    c: Vec4,
    material: Material,
}

impl Triangle {
    pub fn new_lambertian(a: Vec3, b: Vec3, c: Vec3, color: Vec3) -> Self {
        Self {
            a: a.extend(1.0),
            b: b.extend(1.0),
            c: c.extend(1.0),
            material: Material::new_lambertian(color),
        }
    }
    pub fn new_metal(a: Vec3, b: Vec3, c: Vec3, color: Vec3, fuzzy: f32) -> Self {
        Self {
            a: a.extend(1.0),
            b: b.extend(1.0),
            c: c.extend(1.0),
            material: Material::new_metal(color, fuzzy),
        }
    }
    pub fn new_dielectric(a: Vec3, b: Vec3, c: Vec3, ir: f32) -> Self {
        Self {
            a: a.extend(1.0),
            b: b.extend(1.0),
            c: c.extend(1.0),
            material: Material::new_dielectric(ir),
        }
    }
}
