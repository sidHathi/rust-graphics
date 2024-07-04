use cgmath::{
  Point3,
  Vector3
};
use std::hash::{
  Hash, Hasher
};
use std::rc::Rc;

use crate::graphics::ModelVertex;

use super::sdf_shape::{self, SdfShape};

#[derive(Debug, Clone)]
pub struct TriVertex<'a> {
  pub loc: Point3<f32>,
  index: usize,
  closest_vertices: Vec<Option<&'a TriVertex<'a>>>
}

impl<'a> TriVertex<'a> {
  pub fn new(loc: Point3<f32>, index: usize, closest_vertices: Option<Vec<Option<&'a TriVertex>>>) -> TriVertex<'a> {
    // purpose of this is to guarantee that the closest vertex
    // vector is of length 26 (encompasses all possible slots)
    const DEF_ARR_VAL: Option<&'static TriVertex> = None;
    let default_closest_vertices: Vec<Option<&'static TriVertex>> = Vec::from_iter([DEF_ARR_VAL; 26].into_iter());
    let mut closest_vertices_safe = closest_vertices.unwrap_or(default_closest_vertices.clone());
    if closest_vertices_safe.len() < 26 {
      closest_vertices_safe = default_closest_vertices;
    }
    TriVertex {
      loc,
      index,
      closest_vertices: closest_vertices_safe,
    }
  }

  pub fn get_index(&self) -> usize {
    self.index
  }

  pub fn change_index(&mut self, new_idx: usize) {
    self.index = new_idx;
  } 

  pub fn add_neighbor(&mut self, vert_slot: u8, vert: &'a TriVertex) {
    if vert_slot > 25 { return }
    self.closest_vertices[vert_slot as usize] = Some(vert);
  }

  pub fn set_neighbors(&mut self, neighbors: Vec<Option<&'a TriVertex<'a>>>) {
    self.closest_vertices = neighbors;
  }

  pub fn get_neighbor_at_index(&self, idx: usize) -> &Option<&TriVertex<'a>> {
    self.closest_vertices.get(idx).unwrap()
  }

  pub fn into_model_vertex(&self, sdf_shape: &SdfShape) -> ModelVertex {
    // initial implementation -> leave all the texcords at 0, 0
    // populate normal using sdf
    // binormal and bitangent (at least to some extent) are more relevant
    // for depth texture mapping -> not sure if that's necessary right now
    let normal = sdf_shape.compute_normal(self.loc);
    let tex_coords: [f32; 2] = [0.0; 2];
    let tangent: [f32; 3] = [0.0; 3];
    let bitangent: [f32; 3] = [0.0; 3];
    ModelVertex {
      position: self.loc.into(),
      tex_coords,
      normal: normal.into(),
      tangent,
      bitangent
    }
  }

  pub fn get_possible_triangle_list(&self) -> Vec<(usize, usize)> {
    // want all triples of vertices from the closest vertex list that include
    // the current TriVertex
    let mut list: Vec<(usize, usize)> = Vec::new();
    for (idx1, v1_opt) in self.closest_vertices.iter().enumerate() {
      // print!("Checking cv idx {} ", idx1);
      if let Some(v1) = v1_opt {
        // print!("- Found!");
        for (idx2, v2_opt) in self.closest_vertices.iter().enumerate() {
          // print!("Checking cv idx {} ", idx2);
          if let Some(v2) = v2_opt {
            // print!(" - Found!");
            if (idx1 == idx2) {
              continue;
            }
            // check to make sure they're not colinear -> 
            // requires extracting x, y, z idx
            let x_idx_1 = (idx1 as f32 / 9.0).floor() as usize;
            let x_idx_2 = (idx2 as f32 / 9.0).floor() as usize;
            let y_idx_1 = ((idx1 - (9 * x_idx_1)) as f32 / 3.0).floor() as usize;
            let y_idx_2 = ((idx2 - (9 * x_idx_2)) as f32 / 3.0).floor() as usize;
            let z_idx_1 = (idx1 - (9 * x_idx_1) - (3 * y_idx_1)) as usize;
            let z_idx_2 = (idx2 - (9 * x_idx_2) - (3 * y_idx_2)) as usize;

            if (z_idx_1 == 0 && z_idx_2 == 2) || (y_idx_1 == 0 && y_idx_2 == 2) || (x_idx_1 == 0 && x_idx_2 == 2) {
              // skip colinear vertices
              // continue;
            }
            list.push((idx1, idx2))
          }
        }
      }
    }
    list
  }

  fn to_string(&self) -> String {
    let pt_str = format!("loc: {}, {}, {}", self.loc.x, self.loc.y, self.loc.z);
    format!("{}, index: {}, cv_len: {}", pt_str, self.index, self.closest_vertices.len())
  }

  pub fn debug_str(&self) -> String {
    let mut out: String = String::new();
    out += "TriVertex: ";
    out += self.to_string().as_str();
    return out;
  }
}

impl<'a> PartialEq for TriVertex<'a> {
  fn eq(&self, other: &Self) -> bool {
      self.loc.x == other.loc.x && self.loc.y == other.loc.y && self.loc.z == other.loc.z
  }
}

impl<'a> Eq for TriVertex<'a> {}

impl<'a> Hash for TriVertex<'a> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.to_string().hash(state)
  }
}

#[derive(Clone)]
pub struct Triangle<'a> {
  pub a: TriVertex<'a>,
  pub b: TriVertex<'a>,
  pub c: TriVertex<'a>,
}

impl<'a> Triangle<'a> {
  pub fn new(a: TriVertex<'a>, b: TriVertex<'a>, c: TriVertex<'a>) -> Triangle<'a> {
    Triangle {
      a,
      b,
      c
    }
  }

  pub fn midpoint(&self) -> Point3<f32> {
    Point3 { 
      x: (self.a.loc.x + self.b.loc.x + self.c.loc.x) / 3.0, 
      y: (self.a.loc.y + self.b.loc.y + self.c.loc.y) / 3.0, 
      z: (self.a.loc.z + self.b.loc.z + self.c.loc.z) / 3.0
    }
  }

  pub fn hash_str(&self) -> String {
    let midpoint = self.midpoint();
    format!("x: {}, y: {}, z: {}", midpoint.x, midpoint.y, midpoint.z)
  }

  pub fn exact_eq(&self, other: &Self) -> bool {
    if self.a == other.a && self.b == other.b && self.c == other.c {
      return true;
    }
    false
  }

  pub fn face_normal(&self) -> Vector3<f32> {
    let v1 = self.b.loc - self.a.loc;
    let v2 = self.c.loc - self.a.loc;
    v1.cross(v2)
  }

  pub fn debug_str(&self) -> String {
    let mut out = String::new();
    out += format!("Triangle: a: {}, b: {}, c: {};", self.a.debug_str(), self.b.debug_str(), self.c.debug_str()).as_str();
    return out;
  }
}

impl<'a, 'b> PartialEq for Triangle<'a> {
  fn eq(&self, other: &Self) -> bool {
    // two triangle is equal if some rotation of the vertices of one triangle equals the other
    let cmp_vertices = |tri: &Triangle, arr: &[&TriVertex; 3]| -> bool {
      if tri.a.loc == arr[0].loc && tri.b.loc == arr[1].loc && tri.c.loc == arr[2].loc {
        return true;
      }
      false
    };
    let rot1 = &[&other.a, &other.b, &other.c];
    let rot2 = &[&other.b, &other.c, &other.a];
    let rot3 = &[&other.c, &other.a, &other.b];
    cmp_vertices(self, rot1) || cmp_vertices(self, rot2) || cmp_vertices(self, rot3)
  }
}

impl<'a, 'b> Eq for Triangle<'a> {}

pub struct Quad<'a> {
  a: TriVertex<'a>,
  b: TriVertex<'a>,
  c: TriVertex<'a>,
  d: TriVertex<'a>,
}

trait Face {
  fn face_normal(&self) -> Vector3<f32>;
  fn midpoint(&self) -> Point3<f32>;
}

impl<'a> Face for Triangle<'a> {
  fn face_normal(&self) -> Vector3<f32> {
    todo!();
  }

  fn midpoint(&self) -> Point3<f32> {
    todo!();
  }
}

impl<'a> Face for Quad<'a> {
  fn face_normal(&self) -> Vector3<f32> {
    todo!();
  }

  fn midpoint(&self) -> Point3<f32> {
    todo!();
  }
}