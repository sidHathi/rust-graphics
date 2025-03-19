

use cgmath::{
  ortho, Matrix4, Point3, Vector3
};


use wgpu::util::DeviceExt;

use crate::{engine::utils::Positioned, graphics::{InstanceRaw, ModelVertex, Texture, Vertex}};

use super::{light_shadow_uniform::LightShadowUniform, shadow_map_pipeline::get_shadow_pipeline, utils::readback_shadows};

pub struct DirectionalLight {
  pub pos: Point3<f32>,
  pub dims: Vector3<f32>,
  pub dir: Vector3<f32>,
  pub uniform: LightShadowUniform,
  pub buffer: wgpu::Buffer,
  pub light_shadow_bind_group_layout: wgpu::BindGroupLayout,
  pub light_shadow_bind_group: wgpu::BindGroup,
  pub shadow_dt: Texture,
  pub shadow_map_pipeline: wgpu::RenderPipeline,
  pub shadow_dt_bind_group_layout: wgpu::BindGroupLayout,
  pub shadow_dt_bind_group: wgpu::BindGroup,
}

impl DirectionalLight {
  pub fn new(
    pos: Point3<f32>,
    dir: Vector3<f32>,
    dims: Vector3<f32>,
    color: Vector3<f32>,
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
  ) -> Self {
    let shadow_dt = Texture::create_depth_texture(device, config, "shadow map");

    let light_shadow_uniform = LightShadowUniform::new_directional(pos, dir, dims, color);

    let light_shadow_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("Light Shadow Buffer"),
        contents: bytemuck::cast_slice(&[light_shadow_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      }
    );
    let light_shadow_bind_group_layout = device.create_bind_group_layout(
      &wgpu::BindGroupLayoutDescriptor {
        label: Some("light_shadow_bind_group_layout"),
        entries : &[
          wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
              ty: wgpu::BufferBindingType::Uniform,
              has_dynamic_offset: false,
              min_binding_size: None,
            },
            count: None,
          }
        ]
      }
    );
    let light_shadow_bind_group = device.create_bind_group(
      &wgpu::BindGroupDescriptor {
        layout: &light_shadow_bind_group_layout,
        label: Some("light_uniforms_bind_group"),
        entries: &[
          wgpu::BindGroupEntry {
            binding: 0,
            resource: light_shadow_buffer.as_entire_binding(),
          }
        ]
      }
    );
    
    let shadow_render_pipeline = {
      let shadow_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Shadow render pipeline layout"),
        bind_group_layouts: &[&light_shadow_bind_group_layout],
        push_constant_ranges: &[],
      });

      let shader = wgpu::ShaderModuleDescriptor {
        label: Some("light shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shadow_mapper.wgsl").into())
      };

      get_shadow_pipeline(
        device, 
        &shadow_pipeline_layout, 
        Some(Texture::DEPTH_FORMAT), 
        &[ModelVertex::desc(), InstanceRaw::desc()], 
        shader, 
        "vs_main"
      )
    };

    let shadow_dt_bind_group_layout = device.create_bind_group_layout(
      &wgpu::BindGroupLayoutDescriptor {
        label: Some("shadow_dt_bind_group_layout"),
        entries : &[
          wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture { 
              sample_type: wgpu::TextureSampleType::Depth, 
              view_dimension: wgpu::TextureViewDimension::D2, 
              multisampled: false
            },
            count: None
          },
          wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
            count: None,
          }
        ]
      }
    );

    let shadow_dt_bind_group = device.create_bind_group(
      &wgpu::BindGroupDescriptor {
        label: Some("shadow_dt_bind_group"),
        layout: &shadow_dt_bind_group_layout,
        entries: &[
          wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&shadow_dt.view)
          },
          wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::Sampler(&shadow_dt.sampler)
          }
        ]
      }
    );

    Self {
      pos,
      dir,
      dims,
      uniform: light_shadow_uniform,
      buffer: light_shadow_buffer,
      light_shadow_bind_group_layout,
      light_shadow_bind_group,
      shadow_dt,
      shadow_dt_bind_group,
      shadow_dt_bind_group_layout,
      shadow_map_pipeline: shadow_render_pipeline
    }
  }

  pub fn shadow_render_pass(&self) -> wgpu::RenderPassDescriptor {
    wgpu::RenderPassDescriptor {
      color_attachments: &[],
      label: Some("Shadow map render pass"),
      depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
        view: &self.shadow_dt.view,
        depth_ops: Some(wgpu::Operations {
            load: wgpu::LoadOp::Clear(1.0),
            store: wgpu::StoreOp::Store,
        }),
        stencil_ops: None,
      }),
      timestamp_writes: None,
      occlusion_query_set: None
    }
  }

  pub fn view_proj(&self) -> Matrix4<f32> {
    let view = Matrix4::look_to_rh(
      self.pos,
      self.dir, 
      Vector3::unit_y()
    );
    let proj = ortho(-self.dims.x, self.dims.x, -self.dims.y, self.dims.y, -self.dims.z, self.dims.z);
    proj * view
  }

  pub fn readback_shadows(
    &self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    config: &wgpu::SurfaceConfiguration,
  ) {
    readback_shadows(&self.shadow_dt, device, queue, config)
  }
}

impl Positioned for DirectionalLight {
  fn get_position(&self) -> Option<Point3<f32>> {
    Some(self.pos)
  }
}