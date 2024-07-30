use std::{collections::HashMap, sync::{Arc, Mutex}};

use cgmath::{num_traits::abs, EuclideanSpace, Matrix4, Point3, Quaternion, SquareMatrix, Transform, Vector3};

use crate::{engine::{component_store::ComponentKey, raycasting::Ray, transforms::ColliderTransform}, sdf::SdfShape};

pub const NORMAL_TOL: f32 = 0.01;

pub trait ColliderBoundary: Send + Sync {
  fn closest_boundary_pt(&self, pt: Point3<f32>) -> Point3<f32>;
  fn is_interior_point(&self, pt: Point3<f32>) -> bool;
  fn get_boundary_normal(&self, pt: Point3<f32>, tol: f32) -> Option<Vector3<f32>>;
  fn center(&self) -> Point3<f32>;
  fn ray_intersect(&self, ray: &Ray, max_dist: f32) -> Option<Point3<f32>>;
}


#[derive(Clone, Copy, Debug)]
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
    // needs to transform the point into own coord system and then find closest
    if let Some(t_mat) = self.transform.to_coord_matrix().invert() {
      let transformed = t_mat.transform_point(pt);
      return self.underlying.lock().unwrap().closest_boundary_pt(transformed)
    }
    self.underlying.lock().unwrap().closest_boundary_pt(pt)
  }

  pub fn collide(&self, other: &Collider) -> Option<Collision> {
    let mut center = self.underlying.lock().unwrap().center().to_vec() + self.transform.relative_pos;
    if let Some(global_transform) = self.transform.get_global_transform() {
      // println!("Collider global transform: {:?}", global_transform);
      center = self.underlying.lock().unwrap().center().to_vec() + global_transform.pos;
    }
    let closest = other.closest_boundary_pt(Point3::from_vec(center));
    // closest point has to be transformed into collider space ofc
    let local_pos = self.get_collider_coord_matrix().transform_point(closest);
    if self.underlying.lock().unwrap().is_interior_point(local_pos) {
      let normal = self.underlying.lock().unwrap().get_boundary_normal(closest, NORMAL_TOL);
      return Some(Collision {
        loc: closest,
        normal,
        colliders: (self.index, other.index)
      })
    }
    None
  }

  pub fn get_collider_coord_matrix(&self) -> Matrix4<f32> {
    if let Some(transform_matrix) = self.transform.to_coord_matrix().invert() {
      return transform_matrix
    }
    Matrix4::identity()
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

  pub fn intersects_ray(&self, ray: &Ray, max_dist: f32) -> Option<Point3<f32>> {
    self.underlying.lock().unwrap().ray_intersect(ray, max_dist)
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

  pub fn update_transform(&mut self, new_pos: Vector3<f32>, new_rot: Quaternion<f32>) {
    self.transform.update_transform(new_pos, new_rot);
  }

  pub fn update_pos(&mut self, new_pos: Vector3<f32>) {
    self.transform.update_pos(new_pos);
  }

  pub fn update_rot(&mut self, new_rot: Quaternion<f32>) {
    self.transform.update_rot(new_rot);
  }
}