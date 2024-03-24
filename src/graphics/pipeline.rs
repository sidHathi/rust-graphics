// use super::vertex::Vertex;
use super::instance::InstanceRaw;
use super::texture::Texture;
use super::model::{
  Vertex,
  ModelVertex
};

pub fn get_render_pipeline(
  device: &wgpu::Device, 
  config: &wgpu::SurfaceConfiguration, 
  texture_bind_group_layout: &wgpu::BindGroupLayout, 
  camera_bind_group_layout: &wgpu::BindGroupLayout,
  vert_entry: &str, frag_entry: &str
) -> wgpu::RenderPipeline {
  let shader = device.create_shader_module(wgpu::include_wgsl!("../shader.wgsl"));
  let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    label: Some("Render Pipeline Layout"),
    bind_group_layouts: &[
      texture_bind_group_layout,
      camera_bind_group_layout,
    ],
    push_constant_ranges: &[],
  });

  device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    label: Some("Render Pipeline"),
    layout: Some(&render_pipeline_layout),
    vertex: wgpu::VertexState {
      module: &shader,
      entry_point: vert_entry, // 1.
      buffers: &[
        ModelVertex::desc(),
        InstanceRaw::desc(),
      ], // 2.
    },
    fragment: Some(wgpu::FragmentState { // 3.
      module: &shader,
      entry_point: frag_entry,
      targets: &[Some(wgpu::ColorTargetState { // 4.
        format: config.format,
        blend: Some(wgpu::BlendState::REPLACE),
        write_mask: wgpu::ColorWrites::ALL,
      })],
    }),
    primitive: wgpu::PrimitiveState { 
      topology: wgpu::PrimitiveTopology::TriangleList, 
      strip_index_format: None, 
      front_face: wgpu::FrontFace::Ccw, 
      cull_mode: Some(wgpu::Face::Back), 
      unclipped_depth: false, 
      polygon_mode: wgpu::PolygonMode::Fill, 
      conservative: false,
    },
    depth_stencil: Some(wgpu::DepthStencilState {
      format: Texture::DEPTH_FORMAT,
      depth_write_enabled: true,
      depth_compare: wgpu::CompareFunction::Less, // 1.
      stencil: wgpu::StencilState::default(), // 2.
      bias: wgpu::DepthBiasState::default(),
    }),
    multisample: wgpu::MultisampleState {
      count: 1,
      mask: !0,
      alpha_to_coverage_enabled: false
    },
    multiview: None,
  })
}
