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
    pub fn new(bound_min: Vec4, bound_max: Vec4) -> Self {
        Self {
            bound_min,
            bound_max,
        }
    }
    pub fn union(&mut self, xyzw: [f32; 4]) {
        self.refit(Vec4::from_array(xyzw))
    }
    pub fn refit(&mut self, vertex: Vec4) {
        self.bound_min = self.bound_min.min(vertex.clone());
        self.bound_max = self.bound_max.max(vertex);
    }
}
