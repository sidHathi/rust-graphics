use cgmath::Point3;

pub trait Positioned: Send + Sync {
  fn get_position(&self) -> Option<Point3<f32>>;
}
