use glam::{Vec4, Vec4Swizzles};

use crate::geometry::Mesh;
use crate::scene::bvh::Node;
use crate::scene::bvh::Triangle;
use crate::scene::material::Material;
use std::cmp::{min, Ordering};
use std::collections::VecDeque;

#[derive(Debug, Default)]
pub struct Tree {
    pub sizes: [u32; 2],
    pub nodes: Vec<Node>,
    pub triangles: Vec<Triangle>,
    pub materials: Vec<Material>,
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
            triangles: Vec::new(),
            nodes: Vec::new(),
            materials: Vec::new(),
            sizes: [0, 0],
        }
    }
    pub fn build(&mut self) {
        let mut q = VecDeque::new();
        let m = self.triangles.len();
        let n = m.next_power_of_two();
        q.push_back((0, n, 0));
        while let Some((i, j, depth)) = q.pop_front() {
            let l = i;
            let r = min(j, m);
            if l + 1 >= r {
                continue;
            }
            // println!("traverse {i}~{j} depth {depth}");
            self.triangles[l..r].sort_by(|a, b| {
                a.custom[depth % 3]
                    .partial_cmp(&b.custom[depth % 3])
                    .unwrap_or(Ordering::Equal)
            });
            let m = (i + j) / 2;
            q.push_back((i, m, depth + 1));
            q.push_back((m, j, depth + 1));
        }
        self.nodes = vec![Node::default(); n];
        for (i, t) in self.triangles.iter().enumerate() {
            let mut j = (i + n) / 2;
            while j > 0 {
                self.nodes[j].union(t.a);
                self.nodes[j].union(t.b);
                self.nodes[j].union(t.c);
                j /= 2;
            }
        }
        for t in self.triangles.iter_mut() {
            let normal = (t.b - t.a).xyz().cross((t.c - t.a).xyz());
            t.custom = normal.normalize();
        }
        self.sizes = [n as u32, m as u32];
    }
    pub fn add_mesh(&mut self, mesh: Mesh) {
        let material = self.materials.len() as u32;
        self.materials.push(mesh.material);
        self.triangles.extend(mesh.indices.chunks_exact(3).map(|t| {
            let a = Vec4::from_array(mesh.vertices[t[0] as usize].position);
            let b = Vec4::from_array(mesh.vertices[t[1] as usize].position);
            let c = Vec4::from_array(mesh.vertices[t[2] as usize].position);
            let center3x = (a + b + c).xyz();
            Triangle {
                a,
                b,
                c,
                material,
                custom: center3x,
            }
        }));
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
        let mut tree: Tree = mesh.into();
        tree.build();
        assert_eq!(tree.sizes, [16, 12]);
        assert_eq!(tree.nodes.len(), 16);
        assert_eq!(tree.triangles.len(), 12);
        assert_eq!(tree.materials.len(), 1);
    }

    #[test]
    fn suzanne() {
        let mesh = Mesh::load_obj(
            include_bytes!("../../assets/suzanne.obj"),
            Material::new_lambertian(Vec3::new(0.5, 0.5, 0.5)),
        );
        let mut tree: Tree = mesh.into();
        tree.build();
        assert_eq!(tree.sizes, [1024, 979]);
        assert_eq!(tree.nodes.len(), 1024);
        assert_eq!(tree.triangles.len(), 979);
        assert_eq!(tree.materials.len(), 1);
    }
}
