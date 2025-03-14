use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Camera {
    eye: Vec4,
    direction: Vec4,
    up: Vec4,
    right: Vec4,
    params: Vec4,
}

impl Camera {
    pub fn new(from: Vec3, to: Vec3, focal_length: f32, focal_blur_amount: f32, fov: f32) -> Self {
        let eye = from;
        let direction = (to - from).normalize();
        let right = direction.cross(Vec3::Y).normalize();
        let up = right.cross(direction).normalize();
        println!("camera, eye = {eye:?}, dir {direction:?} up {up:?} right {right:?}");
        Self {
            eye: eye.extend(1.0),
            direction: direction.extend(1.0),
            up: up.extend(1.0),
            right: right.extend(1.0),
            params: Vec4::new(focal_length, focal_blur_amount, fov, 0.0),
        }
    }
}
