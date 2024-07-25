use cgmath::{Vector3, Quaternion};

mod component_transform;
mod model_transform;
mod collider_transform;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum TransformType {
  Global,
  Local
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct GlobalTransform {
  pub pos: Vector3<f32>,
  pub rot: Quaternion<f32>
}

pub use component_transform::ComponentTransform;
pub use model_transform::ModelTransform;
pub use collider_transform::ColliderTransform;