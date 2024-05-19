use bytemuck::Pod;
use bytemuck::Zeroable;
use glam::Vec3;
use glam::Vec4;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
pub struct Triangle {
    pub a: Vec4,
    pub b: Vec4,
    pub c: Vec4,
    pub custom: Vec3,
    pub material: u32,
}
