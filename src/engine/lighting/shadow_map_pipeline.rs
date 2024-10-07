pub fn get_shadow_pipeline(
  device: &wgpu::Device, 
  render_pipeline_layout: &wgpu::PipelineLayout,
  depth_format: Option<wgpu::TextureFormat>,
  vertex_layouts: &[wgpu::VertexBufferLayout],
  shader: wgpu::ShaderModuleDescriptor,
  vert_entry: &str,
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
    fragment: None,
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
