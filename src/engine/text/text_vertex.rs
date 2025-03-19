

use crate::graphics::Vertex;
use std::mem;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct TextVertex {
  pub position: [f32; 3],
  pub tex_coords: [f32; 2]
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct TextModelData {
  pub model_matrix: [[f32; 4]; 4],
  pub color: [f32; 3],
  pub opacity: f32
}

impl TextVertex {
  pub fn placeholder() -> TextVertex {
    TextVertex {
      position: [0.; 3],
      tex_coords: [0.; 2]
    }
  }
}

impl Vertex for TextVertex {
  fn desc() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
      array_stride: mem::size_of::<TextVertex>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &[
        wgpu::VertexAttribute {
          offset: 0,
          shader_location: 0,
          format: wgpu::VertexFormat::Float32x3,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
          shader_location: 1,
          format: wgpu::VertexFormat::Float32x2,
        },
      ]
    }
  }
}

impl Vertex for TextModelData {
  fn desc() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
      array_stride: mem::size_of::<TextModelData>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Instance,
      attributes: &[
        wgpu::VertexAttribute {
          offset: 0,
          shader_location: 2,
          format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
          shader_location: 3,
          format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
          shader_location: 4,
          format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
          shader_location: 5,
          format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
          shader_location: 6,
          format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
          shader_location: 7,
          format: wgpu::VertexFormat::Float32,
        },
      ]
    }
  }
}