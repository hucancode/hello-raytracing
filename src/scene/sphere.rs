use crate::scene::material::Material;
use glam::Vec3;
struct Sphere {
  center: Vec3,
  color: Vec3,
  radius: f32,
  material: Material,
}