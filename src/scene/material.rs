use glam::Vec3;
pub enum MaterialKind {
  Lambertian,
  Metal,
  Dielectric,
}
pub struct Material {
  kind: MaterialKind,
  albedo: Vec3,
  param1: f32,
}