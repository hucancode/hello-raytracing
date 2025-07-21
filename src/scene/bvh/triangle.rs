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

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
pub struct CompactTriangle {
    pub v0: [f32; 3],
    pub material: u32,
    pub v1: [f32; 3],
    pub pad1: u32,
    pub v2: [f32; 3],
    pub pad2: u32,
}
