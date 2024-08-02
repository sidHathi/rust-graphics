use core::f32;

use cgmath::{EuclideanSpace, InnerSpace, Matrix3, Matrix4, MetricSpace, Point3, Transform, Vector2, Vector3};

use crate::sdf::SdfShape;

const MAX_ST_ITERS: u32 = 1000;
const DEFAULT_ST_TOL: f32 = 1e-4;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ray {
  pub origin: Point3<f32>,
  pub direction: Vector3<f32>
}

impl Ray {
  pub fn new(origin: Point3<f32>, direction: Vector3<f32>) -> Self {
    Self {
      origin,
      direction
    }
  }

  pub fn gen_ortho(
    screen_coord: Vector2<f32>, 
    eye: Vector3<f32>,
    u: Vector3<f32>,
    v: Vector3<f32>,
    w: Vector3<f32>
  ) -> Self {
    let origin = eye + screen_coord.x * u + screen_coord.y * v;
    let direction = (-1. * w).normalize();
    Self {
      origin: Point3::from_vec(origin),
      direction
    }
  }

  pub fn gen_perspective(
    screen_coord: Vector2<f32>, 
    eye: Vector3<f32>,
    u: Vector3<f32>,
    v: Vector3<f32>,
    w: Vector3<f32>,
    focal_len: f32,
  ) -> Self {
    let origin = eye;
    let direction = ((-focal_len * w) + screen_coord.x * u + screen_coord.y * v).normalize();
    Self {
      origin: Point3::from_vec(origin),
      direction
    }
  }

  pub fn sphere_trace(
    &self, 
    sdf: &SdfShape, 
    max_dist: Option<f32>,
    max_iter: Option<u32>,
    tol: Option<f32>,
    iters: &mut u32
  ) -> Option<Point3<f32>> {
    *iters = 1;
    let mut loc: Point3<f32> = self.origin;
    let mut step_size: f32;
    while *iters < max_iter.unwrap_or(MAX_ST_ITERS) && self.origin.distance(loc) < max_dist.unwrap_or(f32::INFINITY) {
        step_size = sdf.dist(loc);
        loc = loc + self.direction * step_size;
        if step_size < tol.unwrap_or(DEFAULT_ST_TOL) {
            return Some(loc)
        }
        *iters += 1;
    }
    return None;
  }

  pub fn get_transformed(&self, transform_mat: Matrix4<f32>) -> Self {
    Self {
      origin: transform_mat.transform_point(self.origin),
      direction: transform_mat.transform_vector(self.direction),
    }
  }
}