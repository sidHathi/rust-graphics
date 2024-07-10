use cgmath::{Matrix, Matrix3, Matrix4, Point3, Quaternion, SquareMatrix, Vector3};

use super::transforms::{ComponentTransform, ModelTransform, TransformType};

use crate::graphics::Instance;
use cgmath::Transform;

pub struct TransformQueue {
  queue: Vec<ComponentTransform>
}

impl TransformQueue {
  pub fn new() -> TransformQueue {
    Self {
      queue: Vec::new()
    }
  }

  pub fn push(&mut self, transform: ComponentTransform) {
    // println!("Adding transform");
    self.queue.push(transform)
  }

  pub fn pop(&mut self) -> Option<ComponentTransform> {
    // println!("Removing transform");
    self.queue.pop()
  }

  pub fn get_transform_matrix(&self) -> Matrix4<f32> {
    let mat = self.queue.iter().fold(Matrix4::identity(), |acc, e| acc * e.to_matrix());
    // println!("transform matrix: {:?}", mat);
    mat
  }

  pub fn transform_model(&self, model_transform: &ModelTransform) -> ModelTransform {
    match model_transform.clone() {
      ModelTransform::Instanced { transform_type, instances } => {
        if transform_type == TransformType::Global {
          return model_transform.clone();
        }
        let instances_transformed = instances.iter()
          .map(|i| Instance {
            rotation: apply_transform(self.get_transform_matrix(), i.rotation),
            position: self.get_transform_matrix().transform_vector(i.position)
          })
          .collect::<Vec<Instance>>();
        return ModelTransform::instanced(instances_transformed, transform_type);
      },
      ModelTransform::Single { transform_type, pos, rot } => {
        if transform_type == TransformType::Global {
          return model_transform.clone();
        }
        let rot_transformed = apply_transform(self.get_transform_matrix(), rot);
        let pos_transformed = to_vec(self.get_transform_matrix().transform_point(to_point(pos)));
        // println!("Queue applied transform to single model. initial pos: {:?}, new pos: {:?}", pos, pos_transformed);
        return ModelTransform::Single { transform_type, pos: pos_transformed, rot: rot_transformed }
      }
    }
  }
}

fn apply_transform(transform: Matrix4<f32>, rotation: Quaternion<f32>) -> Quaternion<f32> {
  let rotation_matrix = Matrix3::from(rotation);
  // Extract the upper-left 3x3 submatrix of the transformation matrix
  let upper_left = Matrix3::new(
    transform.x.x, transform.x.y, transform.x.z,
    transform.y.x, transform.y.y, transform.y.z,
    transform.z.x, transform.z.y, transform.z.z,
  );

  // Apply the transformation to the rotation matrix
  let transformed_rotation_matrix = upper_left * rotation_matrix;

  // Convert the transformed rotation matrix back to a quaternion
  Quaternion::from(transformed_rotation_matrix)
}

fn to_point(v: Vector3<f32>) -> Point3<f32> {
  Point3::new(v.x, v.y, v.z)
}

fn to_vec(v: Point3<f32>) -> Vector3<f32> {
  Vector3::new(v.x, v.y, v.z)
}