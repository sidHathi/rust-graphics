use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
  pub position: [f32; 3],
  pub _padding: u32, // uniforms have 4-float (16-byte) spacing
  pub color: [f32; 3],
  pub _padding_2: u32
}

pub fn get_light_buffer(device: &wgpu::Device, uniform: &LightUniform) -> wgpu::Buffer {
  device.create_buffer_init(
    &wgpu::util::BufferInitDescriptor {
      label: Some("Light VB"),
      contents: bytemuck::cast_slice(&[uniform.clone()]),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
    }
  )
}

pub fn get_light_bind_group_info(device: &wgpu::Device, buffer: &wgpu::Buffer) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
  let layout = device.create_bind_group_layout(
    &wgpu::BindGroupLayoutDescriptor {
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Buffer { 
            ty:  wgpu::BufferBindingType::Uniform, 
            has_dynamic_offset: false, 
            min_binding_size: None
          },
          count: None
        }
      ],
      label: None
    }
  );
  
  let bind_group = device.create_bind_group(
    &wgpu::BindGroupDescriptor {
      layout: &layout,
      label: None,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: buffer.as_entire_binding()
        }
      ]
    }
  );

  (layout, bind_group)
}
