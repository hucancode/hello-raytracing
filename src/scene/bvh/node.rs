use bytemuck::{Pod, Zeroable};
use glam::Vec4;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Node {
    bound_min: Vec4,
    bound_max: Vec4,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CompactNode {
    pub bound_min: [f32; 3],
    pub left_first: u32,
    pub bound_max: [f32; 3],
    pub tri_count: u32,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            bound_min: Vec4::MAX,
            bound_max: Vec4::MIN,
        }
    }
}

impl Default for CompactNode {
    fn default() -> Self {
        Self {
            bound_min: [f32::MAX; 3],
            bound_max: [f32::MIN; 3],
            left_first: 0,
            tri_count: 0,
        }
    }
}

impl Node {
    pub fn union(&mut self, vertex: Vec4) {
        self.bound_min = self.bound_min.min(vertex);
        self.bound_max = self.bound_max.max(vertex);
    }
}