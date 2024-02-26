use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4};

pub const LAMBERTIAN: u32 = 1;
pub const METAL: u32 = 2;
pub const DIELECTRIC: u32 = 3;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Material {
    albedo: Vec4,
    kind: u32,
    params: Vec3,
}

impl Material {
    pub fn new_lambertian(albedo: Vec3) -> Self {
        Self {
            kind: LAMBERTIAN,
            albedo: albedo.extend(1.0),
            params: Vec3::ZERO,
        }
    }
    pub fn new_metal(albedo: Vec3, fuzzy: f32) -> Self {
        Self {
            kind: METAL,
            albedo: albedo.extend(1.0),
            params: Vec3::new(fuzzy, 0.0, 0.0),
        }
    }
    pub fn new_dielectric(ir: f32) -> Self {
        Self {
            kind: DIELECTRIC,
            albedo: Vec4::ONE,
            params: Vec3::new(ir, 0.0, 0.0),
        }
    }
}
