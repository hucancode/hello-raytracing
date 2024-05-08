use bytemuck::{Pod, Zeroable};

use crate::geometry::Mesh;
use crate::scene::bvh::Node;
use crate::scene::material::Material;
use std::cmp::{min, Ordering};
use std::collections::VecDeque;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
pub struct Triangle {
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub material: u32,
    pub normal: [f32; 4],
}
#[derive(Debug, Default)]
pub struct Tree {
    pub sizes: [u32; 2],
    pub nodes: Vec<Node>,
    pub triangles: Vec<Triangle>,
    pub materials: Vec<Material>,
    pub vertices: Vec<[f32; 4]>,
}

impl From<Mesh> for Tree {
    fn from(mesh: Mesh) -> Self {
        let mut ret = Self::new();
        ret.add_mesh(mesh);
        ret
    }
}

impl Tree {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            triangles: Vec::new(),
            nodes: Vec::new(),
            materials: Vec::new(),
            sizes: [0, 0],
        }
    }
    pub fn build(&mut self) {
        let mut q = VecDeque::new();
        let mut triangles: Vec<_> = self
            .triangles
            .iter()
            .map(|t| {
                let a = self.vertices[t.a as usize];
                let b = self.vertices[t.b as usize];
                let c = self.vertices[t.c as usize];
                let center = [
                    (a[0] + b[0] + c[0]) / 3.0,
                    (a[1] + b[1] + c[1]) / 3.0,
                    (a[2] + b[2] + c[2]) / 3.0,
                ];
                (t.clone(), center)
            })
            .collect();
        let m = triangles.len();
        let n = m.next_power_of_two();
        q.push_back((0, n, 0));
        while let Some((i, j, depth)) = q.pop_front() {
            let l = i;
            let r = min(j, m);
            if l + 1 >= r {
                continue;
            }
            // println!("traverse {i}~{j} depth {depth}");
            triangles[l..r].sort_by(|(_, a_center), (_, b_center)| {
                a_center[depth % 3]
                    .partial_cmp(&b_center[depth % 3])
                    .unwrap_or(Ordering::Equal)
            });
            let m = (i + j) / 2;
            q.push_back((i, m, depth + 1));
            q.push_back((m, j, depth + 1));
        }
        self.nodes = vec![Node::default(); n];
        for (i, (t, _)) in triangles.iter().enumerate() {
            let mut j = (i + n) / 2;
            while j > 0 {
                self.nodes[j].union(self.vertices[t.a as usize]);
                self.nodes[j].union(self.vertices[t.b as usize]);
                self.nodes[j].union(self.vertices[t.c as usize]);
                j /= 2;
            }
        }
        self.triangles = triangles.into_iter().map(|(t, _)| t).collect();
        self.sizes = [n as u32, m as u32];
    }
    pub fn add_mesh(&mut self, mesh: Mesh) {
        let v_offset = self.vertices.len() as u32;
        let m_offset = self.materials.len() as u32;
        self.vertices
            .extend(mesh.vertices.iter().map(|v| v.position));
        self.triangles
            .extend(mesh.indices.chunks(3).map(|t| Triangle {
                a: t[0] + v_offset,
                b: t[1] + v_offset,
                c: t[2] + v_offset,
                material: m_offset,
                normal: mesh.vertices[t[0] as usize].normal,
            }));
        self.materials.push(mesh.material);
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use super::*;

    #[test]
    fn simple_cube() {
        let mesh = Mesh::load_obj(
            include_bytes!("../../assets/cube.obj"),
            Material::new_lambertian(Vec3::new(0.5, 0.5, 0.5)),
        );
        let tree: Tree = mesh.into();
        println!("tree = {tree:?}");
    }
    #[test]
    fn suzanne() {
        let mesh = Mesh::load_obj(
            include_bytes!("../../assets/suzanne.obj"),
            Material::new_lambertian(Vec3::new(0.5, 0.5, 0.5)),
        );
        let tree: Tree = mesh.into();
        println!("tree = {tree:?}");
    }
}
