use cgmath::*;
use wgpu::util::DeviceExt;

use crate::playground::pg_vertex::PgVertex;

pub struct PgCube {
  vertex_buffer: wgpu::Buffer,
  num_vertices: u32,
  index_buffer: wgpu::Buffer,
  num_indices: u32,
}

impl PgCube {
  pub fn new(device: &wgpu::Device, center: Point3<f32>, dim: f32) -> PgCube {
    let half_dim = dim * 0.5;
    let vertices: &[PgVertex] = &[
      PgVertex { loc: [-half_dim + center.x, -half_dim + center.y, -half_dim + center.z], color: [0., 0., 0.] },
      PgVertex { loc: [half_dim + center.x, -half_dim + center.y, -half_dim + center.z], color: [1., 0., 0.] },
      PgVertex { loc: [half_dim + center.x, half_dim + center.y, -half_dim + center.z], color: [1., 1., 0.] },
      PgVertex { loc: [-half_dim + center.x, half_dim + center.y, -half_dim + center.z], color: [0., 1., 0.] },
      PgVertex { loc: [-half_dim + center.x, -half_dim + center.y, half_dim + center.z], color: [0., 0., 1.] },
      PgVertex { loc: [half_dim + center.x, -half_dim + center.y, half_dim + center.z], color: [1., 0., 1.] },
      PgVertex { loc: [half_dim + center.x, half_dim + center.y, half_dim + center.z], color: [1., 1., 1.] },
      PgVertex { loc: [-half_dim + center.x, half_dim + center.y, half_dim + center.z], color: [0., 1., 1.] },
    ];

    let indices: &[u16] = &[
      0, 1, 2, 2, 3, 0,
      4, 7, 6, 6, 5, 4,
      3, 2, 6, 6, 7, 3,
      4, 5, 1, 1, 0, 4,
      1, 5, 6, 6, 2, 1,
      4, 0, 3, 3, 7, 4
    ];


    let vertex_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("Debug vertex buffer"),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
      }
    );
    let num_vertices = vertices.len() as u32;

    let index_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(indices),
        usage: wgpu::BufferUsages::INDEX,
      },
    );
    let num_indices = indices.len() as u32;
    
    PgCube {
      vertex_buffer,
      num_vertices,
      index_buffer,
      num_indices
    }
  }
}

pub trait DrawPgCube<'a> {
 fn draw_cube(
  &mut self,
  cube: &'a PgCube
 );
}

impl<'a, 'b> DrawPgCube<'b> for wgpu::RenderPass<'a> where 'b: 'a {
  fn draw_cube(
    &mut self,
    cube: &'b PgCube
   ) {
      self.set_vertex_buffer(0, cube.vertex_buffer.slice(..));
      self.set_index_buffer(cube.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
      self.draw_indexed(0..cube.num_indices, 0, 0..1);
  }
}