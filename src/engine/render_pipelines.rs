use std::{collections::HashMap, hash::Hash};

use crate::graphics::{get_render_pipeline, Texture};

use super::debug::{get_line_render_pipeline, DebugVertex};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum RenderPipelineKey {
  BasicPipeline,
  DirectionalPipeline,
  LinePipeline,
}

pub struct RenderPipelineData {
  layout: wgpu::PipelineLayout,
  pipeline: wgpu::RenderPipeline
}

pub struct RenderPipelines {
  pub pipeline_map: HashMap<RenderPipelineKey, RenderPipelineData>
}

impl RenderPipelines {
  pub fn new() -> Self {
    Self {
      pipeline_map: HashMap::new()
    }
  }

  pub fn init_render_pipeline(
    &mut self,
    key: RenderPipelineKey,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader_source: wgpu::ShaderSource,
    vert_entry: &str,
    frag_entry: &str,
    label: Option<&str>,
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration
  ) {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { 
      label, 
      bind_group_layouts,
      push_constant_ranges: &[]
    });
    let pipeline = {
      let shader = wgpu::ShaderModuleDescriptor {
        label,
        source: shader_source.into()
      };
      get_render_pipeline(
        device, 
        &layout, 
        config.format, 
        Some(Texture::DEPTH_FORMAT), 
        vertex_layouts, 
        shader, 
        vert_entry, 
        frag_entry
      )
    };

    self.pipeline_map.insert(key, RenderPipelineData {
      layout,
      pipeline
    });
  }

  pub fn init_line_pipeline(
    &mut self,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration
  ) {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Line Render Pipeline Layout"),
      bind_group_layouts: &[
        &camera_bind_group_layout,
      ],
      push_constant_ranges: &[],
    });

    use crate::graphics::Vertex;
    // pipline init/config
    let pipeline = {
      let shader = wgpu::ShaderModuleDescriptor {
          label: Some("Debug Line Shader"),
          source: wgpu::ShaderSource::Wgsl(include_str!("debug/debug_line_shader.wgsl").into()),
      };
      get_line_render_pipeline(
        &device,
        &layout,
        config.format,
        Some(Texture::DEPTH_FORMAT),
        &[DebugVertex::desc()],
        shader,
        "vs_main", 
        "fs_main"
      )
    };

    self.pipeline_map.insert(RenderPipelineKey::LinePipeline, RenderPipelineData {
      layout,
      pipeline
    });
  }

  pub fn get_pipeline(&self, key: &RenderPipelineKey) -> Option<&wgpu::RenderPipeline> {
    if let Some(data) = self.pipeline_map.get(key) {
      return Some(&data.pipeline)
    }
    None
  }
}