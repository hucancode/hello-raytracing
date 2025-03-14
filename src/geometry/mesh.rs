use crate::{geometry::Vertex, scene::Material};
use std::io::BufReader;

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: Material,
}

impl Mesh {
    pub fn load_obj(source: &[u8], material: Material) -> Self {
        let mut reader = BufReader::new(source);
        if let Ok((models, _materials)) = tobj::load_obj_buf(
            &mut reader,
            &tobj::LoadOptions {
                // single_index: true,
                ..Default::default()
            },
            |_matpath| Err(tobj::LoadError::GenericFailure),
        ) {
            let mut vertices = Vec::new();
            let mut indices = Vec::new();
            for model in models {
                let mesh = model.mesh;
                // println!("pos: {:?}", mesh.positions.len());
                // println!("index: {:?}", mesh.normal_indices.len());
                // println!("index: {:?}", mesh.indices.len());
                let offset = vertices.len() as u32;
                let n = mesh.positions.len();
                for i in 0..n / 3 {
                    let i = i * 3;
                    let pos = [
                        mesh.positions[i],
                        mesh.positions[i + 1],
                        mesh.positions[i + 2],
                    ];
                    let nor = if mesh.normals.len() <= i + 2 {
                        [0.0, 0.0, 1.0]
                    } else {
                        [mesh.normals[i], mesh.normals[i + 1], mesh.normals[i + 2]]
                    };
                    let col = 0xffff00ff;
                    vertices.push(Vertex::new(pos, nor, col));
                }
                for i in mesh.indices {
                    indices.push(offset + i);
                }
            }
            Self {
                vertices,
                indices,
                material,
            }
        } else {
            Self {
                vertices: Vec::new(),
                indices: Vec::new(),
                material,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use super::*;

    #[test]
    fn simple_cube() {
        let mesh = Mesh::load_obj(
            include_bytes!("../assets/cube.obj"),
            Material::new_lambertian(Vec3::new(0.5, 0.5, 0.5)),
        );
        assert_eq!(mesh.vertices.len(), 8);
        assert_eq!(mesh.indices.len(), 36);
    }

    #[test]
    fn suzanne() {
        let mesh = Mesh::load_obj(
            include_bytes!("../assets/suzanne.obj"),
            Material::new_lambertian(Vec3::new(0.5, 0.5, 0.5)),
        );
        assert_eq!(mesh.vertices.len(), 515);
        assert_eq!(mesh.indices.len(), 2937);
    }
}
