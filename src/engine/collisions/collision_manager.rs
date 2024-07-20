use std::{collections::HashMap, sync::{Arc, Mutex}};

use cgmath::Matrix4;

use crate::engine::{component::Component, component_store::ComponentKey, transform_queue::{apply_quaternion_transform, to_point, to_vec}, transforms::{ColliderTransform, ComponentTransform}, Scene};

use super::collider::{Collider, ColliderBoundary, Collision};
use cgmath::Transform;

pub struct CollisionManager {
  index_collider_map: HashMap<u32, Arc<Mutex<Collider>>>,
  comp_collider_map: HashMap<ComponentKey, Vec<Arc<Mutex<Collider>>>>,
  collisions: Vec<Collision>,
  next_key: u32,
}

impl CollisionManager {
  pub fn new() -> CollisionManager {
    Self {
      index_collider_map: HashMap::new(),
      comp_collider_map: HashMap::new(),
      collisions: Vec::new(),
      next_key: 0
    }
  }

  pub fn add_component_collider(
    &mut self, 
    boundary: impl ColliderBoundary + 'static, 
    parent: ComponentKey,
    transform: Option<ColliderTransform>
  ) -> Arc<Mutex<Collider>> {
    let collider_idx = self.next_key;
    self.next_key += 1;

    let collider = Collider::new(collider_idx, boundary, parent.clone(), transform);
    let collider_rc = Arc::new(Mutex::new(collider));
    if !self.comp_collider_map.contains_key(&parent) {
      self.comp_collider_map.insert(parent.clone(), Vec::new());
    }
    self.comp_collider_map.get_mut(&parent).unwrap().push(collider_rc.clone());
    self.index_collider_map.insert(collider_idx, collider_rc.clone());
    collider_rc
  }

  pub fn remove_component_colliders(&mut self, comp: ComponentKey) -> Option<Vec<Arc<Mutex<Collider>>>> {
    if let Some(colliders) = self.comp_collider_map.remove(&comp) {
      for col in colliders.iter() {
        let idx = col.lock().unwrap().index;
        self.index_collider_map.remove(&idx);
      }
      return Some(colliders)
    }
    None
  }

  pub fn update_collider_positions(&mut self, position_cache: &HashMap<ComponentKey, Matrix4<f32>>) {
    for (key, colliders) in self.comp_collider_map.iter_mut() {
      if position_cache.contains_key(key) {
        let mat = position_cache.get(key).unwrap();
        for collider in colliders {
          let mut mutex_guard = collider.lock().unwrap();
          let curr_transform = mutex_guard.transform.clone();
          let new_pos = to_vec(mat.transform_point(to_point(curr_transform.relative_pos)));
          let new_rot = apply_quaternion_transform(mat, curr_transform.relative_rot);
          mutex_guard.transform.cache_global_pos(new_pos);
          mutex_guard.transform.cache_global_rot(new_rot);
        }
      }
    }
  }

  pub fn trigger_collision_events(&self, scene: &mut Scene) {
    
  }
}