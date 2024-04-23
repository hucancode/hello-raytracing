use crate::geometry::Mesh;
use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4};
use std::cmp::Ordering;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Triangle {
    a: Vec4,
    b: Vec4,
    c: Vec4,
}

impl PartialOrd for Triangle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let ax = self.a.w;
        let ay = self.b.w;
        let az = self.c.w;
        let bx = other.a.w;
        let by = other.b.w;
        let bz = other.c.w;
        if let Some(order) = ax.partial_cmp(&bx) {
            if order != Ordering::Equal {
                return order;
            }
        }
        if let Some(order) = ay.partial_cmp(&by) {
            if order != Ordering::Equal {
                return order;
            }
        }
        az.partial_cmp(&bz)
    }
}
impl Triangle {
    pub fn new(a: Vec3, b: Vec3, c: Vec3) -> Self {
        let center = a.midpoint(b).midpoint(c);
        Self {
            a: a.extend(center.x),
            b: b.extend(center.y),
            c: c.extend(center.z),
        }
    }
    pub fn triangulate(mesh: &Mesh) -> Vec<Self> {
        let n = mesh.indices.len() / 3;
        let mut ret = Vec::with_capacity(n);
        for i in 0..n {
            let ai = mesh.indices[i * 3];
            let bi = mesh.indices[i * 3 + 1];
            let ci = mesh.indices[i * 3 + 2];
            let a = mesh.vertices[ai];
            let b = mesh.vertices[bi];
            let c = mesh.vertices[ci];
            ret.push(Self::new(a, b, c));
        }
        return ret;
    }
}
