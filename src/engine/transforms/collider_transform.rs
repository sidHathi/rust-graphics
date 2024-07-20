use cgmath::{Matrix4, Quaternion, Vector3};

use crate::engine::component_store::ComponentKey;

use super::GlobalTransform;

#[derive(Clone, Copy)]
pub struct ColliderTransform {
  pub parent: ComponentKey,
  pub relative_pos: Vector3<f32>,
  pub relative_rot: Quaternion<f32>,
  cached_global_pos: Option<Vector3<f32>>,
  cached_global_rot: Option<Quaternion<f32>>
}

impl ColliderTransform {
  pub fn new(
    parent: ComponentKey,
    relative_pos: Vector3<f32>,
    relative_rot: Quaternion<f32>
  ) -> ColliderTransform {
    ColliderTransform {
      parent,
      relative_pos,
      relative_rot,
      cached_global_pos: None,
      cached_global_rot: None
    }
  }

  pub fn cache_global_pos(&mut self, pos: Vector3<f32>) {
    self.cached_global_pos = Some(pos);
  }

  pub fn cache_global_rot(&mut self, rot: Quaternion<f32>) {
    self.cached_global_rot = Some(rot);
  }

  pub fn invalidate_cache(&mut self) {
    self.cached_global_pos = None;
    self.cached_global_rot = None;
  }

  pub fn update_pos(&mut self, pos: Vector3<f32>, rot: Quaternion<f32>) {
    self.relative_pos = pos;
    self.relative_rot = rot;
    self.invalidate_cache();
  }

  pub fn get_global_transform(&self) -> Option<GlobalTransform> {
    if self.cached_global_pos.is_none() || self.cached_global_rot.is_none() {
      return None
    }
    Some(GlobalTransform {
      pos: self.cached_global_pos.unwrap().clone(),
      rot: self.cached_global_rot.unwrap().clone()
    })
  }

  pub fn default(parent: ComponentKey) -> ColliderTransform {
    Self {
      parent,
      relative_pos: Vector3::new(0., 0., 0.),
      relative_rot: Quaternion::new(0., 0., 0., 0.),
      cached_global_pos: None,
      cached_global_rot: None
    }
  }
}