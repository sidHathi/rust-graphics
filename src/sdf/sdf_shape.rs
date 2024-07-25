use std::cmp::max;

use cgmath::{
  num_traits::abs, InnerSpace, MetricSpace, Point3, Vector2, Vector3
};

const EPSILON: f32 = 1e4;
// the assumption is, that in this use case, the sphere trace guess should be almost correct
const DEFAULT_TRACE_ITERS: usize = 1; 

#[derive(Clone)]
pub enum Shape {
  Sphere {
    center: Point3<f32>,
    rad: f32
  },
  Line {
    a: Point3<f32>,
    b: Point3<f32>,
  },
  Cube {
    center: Point3<f32>,
    half_bounds: Vector3<f32>
  },
  Cylinder {
    a: Point3<f32>,
    b: Point3<f32>,
    rad: f32
  },
  Cone {
    a: Point3<f32>,
    b: Point3<f32>,
    rad_a: f32,
    rad_b: f32,
  },
  Custom(Vec<f32>),
}

pub fn SphereSdf(shape: &Shape, point: Point3<f32>) -> f32 {
  match shape {
    Shape::Sphere { center, rad } => {
      point.distance(center.clone()) - rad
    }
    _ => 0.
  }
}

pub fn CubeSdf(shape: &Shape, p: Point3<f32>) -> f32 {
  match shape {
    Shape::Cube { center, half_bounds } => {
      let mut d = 0.;
      if abs(p.x) < half_bounds.x && abs(p.y) < half_bounds.y && abs(p.z) < half_bounds.z {
          return f32::max(f32::max(abs(p.x) - half_bounds.x, abs(p.y) - half_bounds.y) as f32, abs(p.z) - half_bounds.z as f32);
      } else if (abs(p.x) < half_bounds.x) {
          if (abs(p.y) < half_bounds.y) {
              d = abs(p.z) - half_bounds.z;
          } else {
              if (abs(p.z) < half_bounds.z) {
                  d = abs(p.y) - half_bounds.y;
              } else {
                  d = (Vector2::new(abs(p.y) - half_bounds.y, abs(p.z) - half_bounds.z)).magnitude();
              }
          }
      } else if (abs(p.y) <= half_bounds.y) {
          if (abs(p.z) <= half_bounds.z) {
              d = abs(p.x) - half_bounds.x;
          } else {
              d = Vector2::new(abs(p.x) - half_bounds.x, abs(p.z) - half_bounds.z).magnitude();
          }
      } else {
          if (abs(p.z) <= half_bounds.z) {
              d = Vector2::new(abs(p.x) - half_bounds.x, abs(p.y) - half_bounds.y).magnitude();
          } else {
              d = Vector3::new(abs(p.x) - half_bounds.x, abs(p.y) - half_bounds.y, abs(p.z) - half_bounds.z).magnitude();
          }
      }
      d
    }
    _ => 0.
  }
}

#[derive(Clone)]
pub struct SdfShape {
  shape: Shape,
  sdf_fn: fn(&Shape, Point3<f32>) -> f32
}

impl SdfShape {
  pub fn new(shape: Shape, sdf_fn: fn(&Shape, Point3<f32>) -> f32) -> Self {
    SdfShape {
      shape,
      sdf_fn,
    }
  }

  pub fn compute_normal(&self, p: Point3<f32>) -> Vector3<f32> {
    let h: f32 = 1e-4;
    let d0 = (self.sdf_fn)(&self.shape, p);
    let dx = (self.sdf_fn)(&self.shape, p + Vector3::new(h, 0.0, 0.0)) - d0;
    let dy = (self.sdf_fn)(&self.shape, p + Vector3::new(0.0, h, 0.0)) - d0;
    let dz = (self.sdf_fn)(&self.shape, p + Vector3::new(0.0, 0.0, h)) - d0;
    Vector3::new(dx, dy, dz).normalize()
  }

  pub fn dist(&self, p: Point3<f32>) -> f32 {
    return (self.sdf_fn)(&self.shape, p);
  }

  pub fn hit(&self, p: Point3<f32>, tol: f32) -> bool {
    if self.dist(p).abs() < tol {
      return true;
    }
    false
  }

  pub fn gradient_trace(&self, p: Point3<f32>, hit_loc: &mut Point3<f32>, caller_max_iters: Option<usize>, caller_tol: Option<f32>) -> bool {
    // idea -> while the distance to the sdf is less than tol (or default tol),
    // move along the gradient of the sdf towards the sdf boundary by a length
    // self.dist()
    let max_iters = caller_max_iters.unwrap_or(DEFAULT_TRACE_ITERS);
    let tol = caller_tol.unwrap_or(EPSILON);
    let mut iter: usize = 0;
    let mut loc = p.clone();
    let mut hit = false;
    while (!hit && iter < max_iters) {
      let dist = self.dist(p);
      loc = loc + (self.compute_normal(loc) * dist);
      if (dist < tol) {
        hit = true;
        *hit_loc = loc;
        return true;
      }
    }
    false
  }
}