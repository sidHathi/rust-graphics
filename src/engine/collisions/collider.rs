use std::{collections::HashMap, sync::{Arc, Mutex}};

use cgmath::{num_traits::abs, Point3, Vector3};

use crate::{engine::{component_store::ComponentKey, transforms::ColliderTransform}, sdf::SdfShape};

pub const NORMAL_TOL: f32 = 0.01;

pub trait ColliderBoundary {
  fn closest_boundary_pt(&self, pt: Point3<f32>) -> Point3<f32>;
  fn is_interior_point(&self, pt: Point3<f32>) -> bool;
  fn get_boundary_normal(&self, pt: Point3<f32>, tol: f32) -> Option<Vector3<f32>>;
  fn center(&self) -> Point3<f32>;
}


#[derive(Clone, Copy)]
pub struct Collision {
  pub colliders: (u32, u32),
  pub loc: Point3<f32>,
  pub normal: Option<Vector3<f32>>
}

pub struct Collider {
  pub index: u32,
  underlying: Arc<Mutex<dyn ColliderBoundary>>,
  pub parent: ComponentKey,
  collision_map: HashMap<u32, Collision>,
  pub transform: ColliderTransform
}

impl Collider {
  pub fn new(
    index: u32, 
    underlying: impl ColliderBoundary + 'static, 
    parent: ComponentKey, 
    transform: Option<ColliderTransform>
  ) -> Collider {
    Self {
      index,
      underlying: Arc::new(Mutex::new(underlying)),
      parent,
      collision_map: HashMap::new(),
      transform: transform.unwrap_or(ColliderTransform::default(parent))
    }
  }

  pub fn closest_boundary_pt(&self, pt: Point3<f32>) -> Point3<f32> {
    self.underlying.lock().unwrap().closest_boundary_pt(pt)
  }

  pub fn collide(&self, other: &Collider) -> Option<Collision> {
    let center = self.underlying.lock().unwrap().center();
    let closest = other.closest_boundary_pt(center);
    if self.underlying.lock().unwrap().is_interior_point(closest) {
      let normal = self.underlying.lock().unwrap().get_boundary_normal(closest, NORMAL_TOL);
      return Some(Collision {
        loc: closest,
        normal,
        colliders: (self.index, other.index)
      })
    }
    None
  }

  pub fn add_collision(&mut self, col: &Collision) -> Option<Collision> {
    let mut collider_idx = col.colliders.0;
    if col.colliders.0 == self.index {
      collider_idx = col.colliders.1
    } else if col.colliders.1 != self.index {
      return None
    }
    self.collision_map.insert(collider_idx, col.clone());
    Some(col.clone())
  }

  pub fn get_collisions(&self) -> Vec<&Collision> {
    self.collision_map.iter().map(|c| c.1).collect::<Vec<&Collision>>()
  }

  pub fn remove_collision(&mut self, idx: u32) -> Option<Collision> {
    self.collision_map.remove(&idx)
  } 

  pub fn reset_collisions(&mut self) {
    self.collision_map.clear()
  }
}