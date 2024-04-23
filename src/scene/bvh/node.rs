use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Node {
    bound_min: Vec4,
    bound_max: Vec4,
    triangle: usize,
    child: usize,
    next: usize,
}

impl Node {
    pub fn new(vertices: &Vec<Vertex>, v_offset: usize, 
        indices: &Vec<usize>, i_offset: usize, 
        material_idx: usize) -> Self {

    }
}
