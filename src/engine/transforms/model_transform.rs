use cgmath::{Matrix, Matrix4, Quaternion, Rad, Rotation3, Vector3};
use crate::graphics::Instance;
use super::TransformType;

#[derive(Clone, PartialEq, Debug)]
pub struct ModelTransform {
  pub transform_type: TransformType,
  pub pos: Vector3<f32>,
  pub rot: Quaternion<f32>,
}

impl ModelTransform {
  pub fn local(pos: Vector3<f32>, rot: Quaternion<f32>) -> ModelTransform {
    Self {
      transform_type: TransformType::Local,
      pos,
      rot,
    }
  }

  pub fn global(pos: Vector3<f32>, rot: Quaternion<f32>) -> ModelTransform {
    Self {
      transform_type: TransformType::Global,
      pos,
      rot,
    }
  }

  pub fn get_pos(&self) -> Vector3<f32> {
    self.pos
  }

  pub fn get_rot(&self) -> Quaternion<f32> {
    self.rot
  }

  pub fn set_rot(&mut self, new_rot: Quaternion<f32>) {
    self.rot = new_rot;
  }

  pub fn set_pos(&mut self, new_rot: Vector3<f32>) {
    self.pos = new_rot;
  }

  pub fn apply_rot(&mut self, axis: Vector3<f32>, angle: Rad<f32>) {
    self.rot = self.rot * Quaternion::from_axis_angle(axis, angle);
  }

  pub fn default() -> ModelTransform {
    Self {
      transform_type: TransformType::Local,
      pos: Vector3::new(0., 0., 0.),
      rot: Quaternion::new(0., 0., 0., 0.),
    }
  }
}
