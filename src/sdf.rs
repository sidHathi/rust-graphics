mod sdf_shape;
mod triangle;
mod triangle_list;
mod inferred_vertex_mesh;

pub struct SdfBounds {
  pub xmin: f32,
  pub xmax: f32,
  pub ymin: f32,
  pub ymax: f32,
  pub zmin: f32,
  pub zmax: f32,
}
