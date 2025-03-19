use cgmath::{Matrix4, Quaternion, Vector3};

mod component_transform;
mod model_transform;
mod collider_transform;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum TransformType {
  Global,
  Local
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct GlobalTransform {
  pub pos: Vector3<f32>,
  pub rot: Quaternion<f32>
}

impl GlobalTransform {
  pub fn as_matrix(&self) -> Matrix4<f32> {
    let rotation_mat = Matrix4::from(self.rot);
    let translation_mat: Matrix4<f32> = Matrix4::from_translation(self.pos);
    
    translation_mat * rotation_mat
  }
}

pub use component_transform::ComponentTransform;
pub use model_transform::ModelTransform;
pub use collider_transform::ColliderTransform;