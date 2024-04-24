use crate::geometry::Mesh;
use crate::scene::bvh::Node;
use std::cmp::{min, Ordering};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Tree {
    pub nodes: Vec<Node>,
    pub indices: Vec<u32>,
    pub vertices: Vec<[f32; 4]>,
    pub normals: Vec<[f32; 4]>,
}

impl Default for Tree {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            indices: Vec::new(),
            vertices: Vec::new(),
            normals: Vec::new(),
        }
    }
}

impl From<Mesh> for Tree {
    fn from(mesh: Mesh) -> Self {
        let n = mesh.indices.len() / 3;
        let mut triangles: Vec<([f32; 3], [u32; 3])> = (0..n)
            .map(|i| {
                let a = mesh.indices[i * 3];
                let b = mesh.indices[i * 3 + 1];
                let c = mesh.indices[i * 3 + 2];
                let triangle = [a, b, c];
                let a = mesh.vertices[a as usize].position;
                let b = mesh.vertices[b as usize].position;
                let c = mesh.vertices[c as usize].position;
                let center = [
                    (a[0] + b[0] + c[0]) / 3.0,
                    (a[1] + b[1] + c[1]) / 3.0,
                    (a[2] + b[2] + c[2]) / 3.0,
                ];
                (center, triangle)
            })
            .collect();
        let mut q = VecDeque::new();
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
            triangles[l..r].sort_by(|(center_a, _), (center_b, _)| {
                center_a[depth % 3]
                    .partial_cmp(&center_b[depth % 3])
                    .unwrap_or(Ordering::Equal)
            });
            let m = (i + j) / 2;
            q.push_back((i, m, depth + 1));
            q.push_back((m, j, depth + 1));
        }
        let vertices: Vec<[f32; 4]> = mesh.vertices.iter().map(|v| v.position).collect();
        let normals: Vec<[f32; 4]> = triangles
            .iter()
            .map(|(_, t)| mesh.vertices[t[0] as usize].normal)
            .collect();
        let mut nodes = vec![Node::default(); n];
        for (i, (_, t)) in triangles.iter().enumerate() {
            let mut j = (i + n) / 2;
            while j > 0 {
                nodes[j].refit_xyzw(vertices[t[0] as usize]);
                nodes[j].refit_xyzw(vertices[t[1] as usize]);
                nodes[j].refit_xyzw(vertices[t[2] as usize]);
                j /= 2;
            }
        }
        let indices = triangles.into_iter().map(|(_, t)| t).flatten().collect();
        Self {
            vertices,
            indices,
            normals,
            nodes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_cube() {
        let mesh = Mesh::load_obj(include_bytes!("../../assets/cube.obj"));
        let tree: Tree = mesh.into();
        println!("tree = {tree:?}");
    }
    #[test]
    fn suzanne() {
        let mesh = Mesh::load_obj(include_bytes!("../../assets/suzanne.obj"));
        let tree: Tree = mesh.into();
        println!("tree = {tree:?}");
        println!(
            "nodes = {}, tris = {}",
            tree.nodes.len(),
            tree.normals.len()
        );
    }
}
