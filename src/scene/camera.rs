use bytemuck::{Pod, Zeroable};
use glam::Vec3;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Camera {
    eye: Vec3,
    _padding1: f32,
    direction: Vec3,
    _padding2: f32,
    up: Vec3,
    _padding3: f32,
    right: Vec3,
    focus_distance: f32,
    // _padding: [u8; 12],
}

impl Camera {
    pub fn new(from: Vec3, to: Vec3, focus_distance: f32) -> Self {
        let eye = from;
        let direction = (to - from).try_normalize().unwrap_or(Vec3::NEG_Z);
        let up = Vec3::new(0.0, -direction.z, direction.y);
        let right = direction.cross(up);
        Self {
            eye,
            direction,
            up,
            right,
            focus_distance,
            _padding1: 0.0,
            _padding2: 0.0,
            _padding3: 0.0,
        }
    }
}
