use glam::Vec3;
use std::cmp::min;
use std::{f32::consts::PI, mem::size_of, sync::Arc};
use wgpu::BufferBindingType;
use winit::window::Window;

use crate::{geometry::Mesh, renderer::Renderer};

use super::{bvh::Node, bvh::Tree, bvh::Triangle, material::Material, Camera};

const MAX_TRIS: usize = 1000000;
const MAX_MATS: usize = 1000;

pub struct SceneTris {
    pub renderer: Renderer,
    pub camera: Camera,
    pub tris_bvh: Tree,
}

impl SceneTris {
    pub fn write_tree_data(&mut self) {
        let data = [
            (
                bytemuck::cast_slice(&self.tris_bvh.sizes),
                2 * size_of::<u32>(),
            ),
            (
                bytemuck::cast_slice(&self.tris_bvh.nodes),
                MAX_TRIS * size_of::<Node>(),
            ),
            (
                bytemuck::cast_slice(&self.tris_bvh.triangles),
                MAX_TRIS * size_of::<Triangle>(),
            ),
            (
                bytemuck::cast_slice(&self.tris_bvh.materials),
                MAX_MATS * size_of::<Material>(),
            ),
        ];
        for (i, (data, size)) in data.into_iter().enumerate() {
            let n = min(data.len(), size);
            self.renderer.write_buffer(&data[0..n], i);
        }
    }
    async fn make_renderer(window: Arc<Window>) -> Renderer {
        Renderer::new(
            window,
            vec![
                (BufferBindingType::Uniform, 2 * size_of::<u32>() as u64), // bvh tree size
                (
                    BufferBindingType::Storage { read_only: true },
                    (MAX_TRIS * size_of::<Node>()) as u64,
                ), // nodes
                (
                    BufferBindingType::Storage { read_only: true },
                    (MAX_TRIS * size_of::<Triangle>()) as u64,
                ), // triangles
                (
                    BufferBindingType::Storage { read_only: true },
                    (MAX_MATS * size_of::<Material>()) as u64,
                ), // materials
            ],
            include_str!("../shaders/shader_tris.wgsl"),
        )
        .await
    }
    pub async fn new_suzane(window: Arc<Window>) -> Self {
        let mesh = Mesh::load_obj(
            include_bytes!("../assets/suzanne.obj"),
            Material::new_lambertian(Vec3::new(0.3, 0.4, 0.6)),
        );
        let mut tree: Tree = mesh.into();
        let mesh = Mesh::load_obj(
            include_bytes!("../assets/cube_s.obj"),
            Material::new_metal(Vec3::new(0.5, 0.5, 0.6), 0.2),
        );
        tree.add_mesh(mesh);
        let mesh = Mesh::load_obj(
            include_bytes!("../assets/cube_m.obj"),
            Material::new_dielectric(0.1),
        );
        tree.add_mesh(mesh);
        let mesh = Mesh::load_obj(
            include_bytes!("../assets/cube_l.obj"),
            Material::new_lambertian(Vec3::new(0.5, 0.5, 0.6)),
        );
        tree.add_mesh(mesh);
        tree.build();
        let camera = Camera::new(
            Vec3::new(0.0, 2.2, 4.5),
            Vec3::new(0.0, 0.0, -4.5),
            5.6,
            0.0,
            PI * 0.3,
        );
        let renderer = Self::make_renderer(window).await;
        Self {
            renderer,
            camera,
            tris_bvh: tree,
        }
    }
    pub async fn new_cube(window: Arc<Window>) -> Self {
        let mesh = Mesh::load_obj(
            include_bytes!("../assets/cube2.obj"),
            Material::new_lambertian(Vec3::new(0.5, 0.5, 0.6)),
        );
        let mut tree: Tree = mesh.into();
        tree.build();
        let camera = Camera::new(
            Vec3::new(0.0, 2.2, 6.5),
            Vec3::new(0.0, 0.1, -3.0),
            2.2,
            0.0,
            PI * 0.3,
        );
        let renderer = Self::make_renderer(window).await;
        Self {
            renderer,
            camera,
            tris_bvh: tree,
        }
    }
    pub async fn new_quad(window: Arc<Window>) -> Self {
        let mesh = Mesh::load_obj(
            include_bytes!("../assets/quad.obj"),
            Material::new_lambertian(Vec3::new(0.5, 0.5, 0.6)),
        );
        let mut tree: Tree = mesh.into();
        tree.build();
        let camera = Camera::new(
            Vec3::new(0.0, 0.2, 3.5),
            Vec3::new(0.0, 0.1, -3.0),
            2.2,
            0.0,
            PI * 0.3,
        );
        let renderer = Self::make_renderer(window).await;
        Self {
            renderer,
            camera,
            tris_bvh: tree,
        }
    }
}
