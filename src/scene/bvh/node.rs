use bytemuck::{Pod, Zeroable};
use glam::Vec4;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Node {
    bound_min: Vec4,
    bound_max: Vec4,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            bound_min: Vec4::MAX,
            bound_max: Vec4::MIN,
        }
    }
}

impl Node {
    pub fn union(&mut self, vertex: Vec4) {
        self.bound_min = self.bound_min.min(vertex);
        self.bound_max = self.bound_max.max(vertex);
    }
}
