use cgmath::{Point3, Rad, Vector3};
use wgpu::util::DeviceExt;

use crate::{engine::{lighting::shadow_map_pipeline::get_shadow_pipeline, utils::Positioned}, graphics::{InstanceRaw, ModelVertex, Texture, Vertex}};

use super::{light_shadow_uniform::{LightShadowUniform, LightShadowUniformConstructionProps}, utils::readback_shadows};

pub struct PointLight {
  pos: Point3<f32>,
  fovy: Rad<f32>,
  aspect: f32,
  znear: f32,
  zfar: f32,
  pub light_shadow_uniform: LightShadowUniform,
  pub light_shadow_buffer: wgpu::Buffer,
  pub light_shadow_bind_group_layout: wgpu::BindGroupLayout,
  pub light_shadow_bind_group: wgpu::BindGroup,
  shadow_dt: Texture,
  pub shadow_map_pipeline: wgpu::RenderPipeline,
  pub shadow_dt_bind_group_layout: wgpu::BindGroupLayout,
  pub shadow_dt_bind_group: wgpu::BindGroup,
}

pub struct PointLightConstructionProps<'a> {
  pub pos: Point3<f32>,
  pub color: Vector3<f32>,
  pub fovy: Rad<f32>,
  pub width: f32,
  pub height: f32,
  pub znear: f32,
  pub zfar: f32,
  pub pitch: Rad<f32>,
  pub yaw: Rad<f32>,
  pub device: &'a wgpu::Device,
  pub config: &'a wgpu::SurfaceConfiguration
}

impl PointLight {
  pub fn new(construction_props: PointLightConstructionProps) -> Self {
    let PointLightConstructionProps { pos, color, fovy, width, height, znear, zfar, pitch, yaw, device, config } = construction_props;
    let aspect = width / height;
    let light_shadow_uniform = LightShadowUniform::new_perspective(LightShadowUniformConstructionProps { pos, fovy, znear, zfar, aspect, pitch, yaw, color });
    let shadow_dt = Texture::create_depth_texture(device, config, "shadow map");

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
    
    let shadow_map_pipeline = {
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
      fovy,
      aspect,
      znear,
      zfar,
      light_shadow_uniform,
      light_shadow_buffer,
      light_shadow_bind_group_layout,
      light_shadow_bind_group,
      shadow_dt,
      shadow_dt_bind_group,
      shadow_dt_bind_group_layout,
      shadow_map_pipeline,
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

  pub fn readback_shadows(
    &self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    config: &wgpu::SurfaceConfiguration,
  ) {
    readback_shadows(&self.shadow_dt, device, queue, config)
  }
}

impl Positioned for PointLight {
  fn get_position(&self) -> Option<Point3<f32>> {
    Some(self.pos)
  }
}
