use cgmath::{Matrix, Matrix4, Quaternion, Vector3};
use tobj::Model;

use crate::graphics::Instance;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum TransformType {
  Global,
  Local
}

#[derive(Clone, Copy, PartialEq)]
pub struct ComponentTransform {
  pub transform_type: TransformType,
  pub pos: Vector3<f32>,
  pub rot: Quaternion<f32>,
}

impl ComponentTransform {
  pub fn local(pos: Vector3<f32>, rot: Quaternion<f32>) -> ComponentTransform {
    Self {
      transform_type: TransformType::Local,
      pos,
      rot
    }
  }

  pub fn global(pos: Vector3<f32>, rot: Quaternion<f32>) -> ComponentTransform {
    Self {
      transform_type: TransformType::Global,
      pos,
      rot
    }
  }

  pub fn default() -> ComponentTransform {
    Self {
      transform_type: TransformType::Local,
      pos: Vector3::new(0., 0., 0.),
      rot: Quaternion::new(0., 0., 0., 0.)
    }
  }

  pub fn to_matrix(&self) -> cgmath::Matrix4<f32> {
    let rotation_mat = Matrix4::from(self.rot);
    let translation_mat: Matrix4<f32> = Matrix4::from_translation(self.pos);
    let combined = translation_mat * rotation_mat;
    // println!("Rotation matrix: {:?}, Translation: {:?}, Combined: {:?}", rotation_mat, translation_mat, combined);
    combined
  }
}

#[derive(Clone, PartialEq)]
pub enum ModelTransform {
  Single {
    transform_type: TransformType,
    pos: Vector3<f32>,
    rot: Quaternion<f32>,
  },
  Instanced {
    transform_type: TransformType,
    instances: Vec<Instance>,
  }
}

impl ModelTransform {
  pub fn local(pos: Vector3<f32>, rot: Quaternion<f32>) -> ModelTransform {
    ModelTransform::Single {
      transform_type: TransformType::Local,
      pos,
      rot
    }
  }

  pub fn global(pos: Vector3<f32>, rot: Quaternion<f32>) -> ModelTransform {
    ModelTransform::Single {
      transform_type: TransformType::Global,
      pos,
      rot
    }
  }

  pub fn instanced(instances: Vec<Instance>, transform_type: TransformType) -> ModelTransform {
    ModelTransform::Instanced {
      instances,
      transform_type,
    }
  }

  pub fn default() -> ModelTransform {
    ModelTransform::Single { transform_type: TransformType::Local, pos: Vector3::new(0., 0., 0.), rot: Quaternion::new(0., 0., 0., 0.) }
  }
}