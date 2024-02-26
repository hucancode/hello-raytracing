use glam::Vec3;
pub struct Camera {
  eye: Vec3,
  direction: Vec3,
  up: Vec3,
  right: Vec3,
  focus_distance: f32,
}