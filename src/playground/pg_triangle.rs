use cgmath::Point3;

use super::pg_vertex::PgVertex;
use wgpu::util::DeviceExt;

pub struct PgTriangle {
  vertex_buffer: wgpu::Buffer,
  num_vertices: u32,
}

impl PgTriangle {
  pub fn new(device: &wgpu::Device, center: Point3<f32>, dim: f32) -> PgTriangle {
    let vertices: &[PgVertex] = &[
      PgVertex { loc: [0.0, 0.5 * dim, 0.0], color: [1.0, 0.0, 0.0] },
      PgVertex { loc: [-0.5 * dim, -0.5 * dim, 0.0], color: [0.0, 1.0, 0.0] },
      PgVertex { loc: [0.5 * dim, -0.5 * dim, 0.0], color: [0.0, 0.0, 1.0] },
    ];

    let vertex_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("Debug vertex buffer"),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
      }
    );
    let num_vertices = vertices.len() as u32;
    
    Self {
      vertex_buffer,
      num_vertices
    }
  }
}

pub trait DrawPgTriangle<'a> {
  fn draw_triangle(&mut self, tri: &'a PgTriangle);
}

impl<'a, 'b> DrawPgTriangle<'b> for wgpu::RenderPass<'a> where 'b: 'a {
  fn draw_triangle(&mut self, tri: &'b PgTriangle) {
      self.set_vertex_buffer(0, tri.vertex_buffer.slice(..));
      self.draw(0..3, 0..1);
  }
}