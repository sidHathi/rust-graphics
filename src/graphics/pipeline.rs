// use super::vertex::Vertex;
use super::instance::InstanceRaw;
use super::texture::Texture;
use super::model::{
  Vertex,
  ModelVertex
};

pub fn get_render_pipeline(
  device: &wgpu::Device, 
  render_pipeline_layout: &wgpu::PipelineLayout,
  color_format: wgpu::TextureFormat,
  depth_format: Option<wgpu::TextureFormat>,
  vertex_layouts: &[wgpu::VertexBufferLayout],
  shader: wgpu::ShaderModuleDescriptor,
  vert_entry: &str,
  frag_entry: &str,
) -> wgpu::RenderPipeline {
  let shader = device.create_shader_module(shader);

  device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    label: Some("Render Pipeline"),
    layout: Some(&render_pipeline_layout),
    vertex: wgpu::VertexState {
      module: &shader,
      entry_point: vert_entry, // 1.
      buffers: vertex_layouts, // 2.
    },
    fragment: Some(wgpu::FragmentState { // 3.
      module: &shader,
      entry_point: frag_entry,
      targets: &[Some(wgpu::ColorTargetState { // 4.
        format: color_format,
        blend: Some(wgpu::BlendState {
          color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::SrcAlpha,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
          },
          alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
          },
        }),
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
    depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
      format,
      depth_write_enabled: true,
      depth_compare: wgpu::CompareFunction::Less,
      stencil: wgpu::StencilState::default(),
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
