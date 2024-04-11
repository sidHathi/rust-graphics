use cgmath::num_traits::float::FloatCore;
use cgmath::num_traits::Float;
use cgmath::{
  InnerSpace, Point3, Vector3
};
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::cmp::{
  max,
  min
};
use std::rc::Rc;
use std::vec;

use super::triangle::{
  TriVertex,
  Triangle,
};
use super::triangle_list::TriangleSet;
use super::sdf_shape::SdfShape;
use crate::graphics::{
  Mesh,
  ModelVertex,
};
use super::SdfBounds;
use crate::util::{
  PointDict,
  Point,
};

const MAX_NEIGHBOR_OFFSET: usize = 3;

pub struct InferredVertexMesh<'a> {
  triangles: TriangleSet<'a>,
  sdf: SdfShape,
  raw_vertex_dims: Vector3<u128>,
  raw_vertices: Vec<Vec<Vec<Option<TriVertex<'a>>>>>,
  bounds: SdfBounds, // what should this look like? -> x/y/z coord bounds needed ig?
  granularity: f32,
  inferred_mesh: Option<Mesh>,
}

// safely adds a TriVertex to a raw 3d arr
fn add_vert<'a>(vertex_arr: &mut Vec<Vec<Vec<Option<TriVertex<'a>>>>>, vert: TriVertex<'a>, x: usize, y: usize, z: usize) {
  if x < 0 || y < 0 || z < 0 { return }
  if vertex_arr.len() < x || vertex_arr[x].len() < y || vertex_arr[x][y].len() < z { return }
  vertex_arr[x][y][z] = Some(vert);
}

// Vert slots go from:
// -x -> +x; -y -> +y; -z -> +z
// z increments fastest, then y, then x
// index 13 with this method of incrementing
// would yield the location of the center vertex
// to avoid this, we'll decrement indices beyond
// 13 by one
// resultant slot = z + x + y (indices)
fn get_vert_slot(x_idx: usize, y_idx: usize, z_idx: usize, x_j: usize, y_j: usize, z_j: usize) -> u8 {
  // this is just gonna be a big match statement
  let x_off = if x_j == x_idx { 0 } else if x_j > x_idx { 1 } else { -1 } as i8;
  let y_off = if y_j == y_idx { 0 } else if y_j > y_idx { 1 } else { -1 } as i8;
  let z_off = if z_j == z_idx { 0 } else if z_j > z_idx { 1 } else { -1 } as i8;
  match (x_off, y_off, z_off) {
    (-1, -1, -1) => 0,
    (-1, -1, 0) => 1,
    (-1, -1, 1) => 2,
    (-1, 0, -1) => 3,
    (-1, 0, 0) => 4,
    (-1, 0, 1) => 5,
    (-1, 1, -1) => 6,
    (-1, 1, 0) => 7,
    (-1, 1, 1) => 8,
    (0, -1, -1) => 9,
    (0, -1, 0) => 10,
    (0, -1, 1) => 11,
    (0, 0, -1) => 12,
    (0, 0, 0) => 0,
    (0, 0, 1) => 13,
    (0, 1, -1) => 14,
    (0, 1, 0) => 15,
    (0, 1, 1) => 16,
    (1, -1, -1) => 17,
    (1, -1, 0) => 18,
    (1, -1, 1) => 19,
    (1, 0, -1) => 20,
    (1, 0, 0) => 21,
    (1, 0, 1) => 22,
    (1, 1, -1) => 23,
    (1, 1, 0) => 24,
    (1, 1, 1) => 25,
    _ => 0
  }
}

fn get_vertex_neighbors<'a, 'b>(vertex_arr: &'a Vec<Vec<Vec<Option<TriVertex>>>>, vert: &'b TriVertex<'a>, x_idx: usize, y_idx: usize, z_idx: usize) -> Vec<Option<&'b TriVertex<'a>>> {
  // want to get the closest vertex in each direction within a cube
  // of dims 3*granularity for each side
  let mut neighbors_slice: &mut [Option<&TriVertex>; 26] = &mut [None; 26];
  for d in 1..MAX_NEIGHBOR_OFFSET {
    for x_j in max(x_idx - 3 * d, 0)..min(x_idx + 3 * d, vertex_arr.len()) {
      for y_j in max(y_idx - 3 * d, 0)..min(y_idx + 3 * d, vertex_arr[x_j].len()) {
        for z_j in max(z_idx - 3 * d, 0)..min(z_idx + 3 * d, vertex_arr[y_j].len()) {
          // only want outermost vertices for the pass -> so if the x, y, z dif
          // from original indices is not equal to d -> skip
          if !((x_j as i32 - x_idx as i32).abs() as usize == d && (y_j as i32 - y_idx as i32).abs() as usize == d && (z_j as i32 - z_idx as i32).abs() as usize == d) {
            continue;
          }

          // check to make sure that a closer vertex at this relative position
          // has not already been added
          let slot = get_vert_slot(x_idx, y_idx, z_idx, x_j, y_j, z_j);
          if neighbors_slice[slot as usize] != None {
            continue;
          }
          neighbors_slice[slot as usize] = vertex_arr[x_j][y_j][z_j].as_ref();
          // check to make sure the vertex at this point actually exists
        }
      }
    }
  }
  return Vec::from(neighbors_slice)
}

fn populate_all_closest_vertices<'a>(vertex_arr: &'a Vec<Vec<Vec<Option<TriVertex<'a>>>>>) -> Vec<Vec<Vec<Option<TriVertex<'a>>>>> {
  // sliding 3x3x3 window
  let mut neighbors_map: PointDict<Vec<Option<&'a TriVertex<'a>>>> = PointDict::new();
  {
    for (x_idx, plane) in vertex_arr.iter().enumerate() {
      // let mut plane_ref = Rc::new(plane);
      // need reference counters for each of the outer loops potentially
      for (y_idx, row) in plane.iter().enumerate() {
        for (z_idx, vert_opt) in row.iter().enumerate() {
          if let Some(vert) = vert_opt {
            // get the vertex's neighbors
            // add all of them as references in the triangle
            let neighbors = get_vertex_neighbors(vertex_arr, vert, x_idx, y_idx, z_idx);
            neighbors_map.insert(Point3{x: x_idx as f32, y: y_idx as f32, z: z_idx as f32}, neighbors);
          }
        }
      }
    }
  }
  let mut mutated_vec = vertex_arr.clone();
  for (key, val) in neighbors_map.iter() {
    let x_idx = key.0.x as usize;
    let y_idx = key.0.y as usize;
    let z_idx = key.0.z as usize;
    if let Some(vert) = &mut mutated_vec[x_idx][y_idx][z_idx] {
      vert.set_neighbors(val.clone())
    }
  }
  mutated_vec
}

fn compare_normal(sdf_shape: SdfShape, triangle: Triangle, tol: f32) -> bool {
  let tri_center = triangle.midpoint();
  let tri_normal = triangle.face_normal();
  let normal = sdf_shape.compute_normal(tri_center);
  if tri_normal.cross(normal).magnitude() < tol && tri_normal.dot(normal) > 0.0 {
    return true;
  }
  false
}

impl<'a> InferredVertexMesh<'a> {
  pub fn construct(sdf_shape: SdfShape, bounds: SdfBounds, granularity: f32) -> InferredVertexMesh<'a> {
    // this should basically subdivide the bounds into tiny regions of size granularity,
    // then, if the sdf tolerance is within some fraction of the granularity value from the current point, it should generate a new vertex at the nearest point where the sdf function is zero (or just the current point maybe
    // then we want to store the vertices at the granularity index corresponding to its location lol
    // should also store the array coords of each vertex in a separate vector, loop over this vector, and populate the fields of the TriVertices
    // At this point, we can start building triangles rip -> maybe a separate function
    // triangle construction involves looping over each trivertex and constructing possible triangles from the nearest vertices, then adding them to a triangle list
    // once we have a list of triangles, it should be possible to extract vertex and index buffers lol

    let dim_x = ((bounds.xmax - bounds.xmin) / granularity).abs().ceil() as usize;
    let dim_y = ((bounds.ymax - bounds.ymin) / granularity).abs().ceil() as usize;
    let dim_z = ((bounds.zmax - bounds.zmin) / granularity).abs().ceil() as usize;

    // 3d granularity vector -> each should correspond to a tiny cubic subdivision of the shape space
    // these subdivisions can basically model points within some error boundary of the center of the cubic region
    let mut vec_3d: Vec<Vec<Vec<Option<TriVertex>>>> = Vec::new();
    for x in 0..dim_x {
      let mut y_arr: Vec<Vec<Option<TriVertex>>> = Vec::new();
      for y in 0..dim_y {
        let mut z_arr: Vec<Option<TriVertex>> = Vec::new();
        for z in 0..dim_z {
          z_arr.push(None);
        }
        y_arr.push(z_arr);
      }
      vec_3d.push(y_arr);
    }

    for x_idx in 0..dim_x {
      for y_idx in 0..dim_y {
        for z_idx in 0..dim_z {
          // At this point we need to infer the coordinates of the cell
          // in the 3d vec based on the sdf bounds and then evaluate the
          // sdf to see if the cell is a "hit"
          let x = (x_idx as f32 * granularity) + bounds.xmin;
          let y = (y_idx as f32 * granularity) + bounds.ymin;
          let z = (z_idx as f32 * granularity) + bounds.zmin;

          let p = Point3 {
            x, y, z
          };
          let tol = granularity / 2.0;
          if sdf_shape.hit(p, tol) {
            // if the point is within the tol distance from the sdf boundary,
            // -> ideally we would evaluate the point on the sdf boundary where the point is zero? -> 
            let mut sdf_loc = p.clone();
            sdf_shape.gradient_trace(p, &mut sdf_loc, None, None);
            let vert = TriVertex::new(sdf_loc, None);
            add_vert(&mut vec_3d, vert, x_idx, y_idx, z_idx);
          }
        }
      }
    }
    
    let completed_arr = populate_all_closest_vertices(&vec_3d);

    todo!()
  }
}
