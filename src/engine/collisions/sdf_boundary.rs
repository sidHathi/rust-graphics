use cgmath::{num_traits::abs, Point3, Vector3};

use crate::{engine::transforms::ComponentTransform, sdf::SdfShape};

use super::collider::ColliderBoundary;

pub struct SdfBoundary {
  pub center: Point3<f32>,
  pub sdf: SdfShape
}

impl ColliderBoundary for SdfBoundary {
  fn closest_boundary_pt(&self, pt: Point3<f32>) -> Point3<f32> {
    let mut hit_loc: Point3<f32> = Point3::new(0., 0., 0.);
    self.sdf.gradient_trace(pt, &mut hit_loc, None, None);
    hit_loc
  }

  fn is_interior_point(&self, pt: Point3<f32>) -> bool {
    // println!("Checking interior point {:?}, center: {:?} -> dist = {}", pt, self.center, self.sdf.dist(pt));
    self.sdf.dist(pt) <= 0.
  }

  fn center(&self) -> Point3<f32> {
    self.center.clone()
  }

  fn get_boundary_normal(&self, pt: Point3<f32>, tol: f32) -> Option<Vector3<f32>> {
    let dist = self.sdf.dist(pt);
    if abs(dist) <= tol {
      return Some(self.sdf.compute_normal(pt))
    }
    None
  }

  fn ray_intersect(&self, ray: &crate::engine::raycasting::Ray, max_dist: f32) -> Option<Point3<f32>> {
    let mut iters: u32 = 0;
    ray.sphere_trace(&self.sdf, Some(max_dist), None, None, &mut iters)
  }
}

impl SdfBoundary {
  pub fn new(center: Point3<f32>, sdf: SdfShape) -> SdfBoundary {
    Self {
      center,
      sdf
    }
  }
}