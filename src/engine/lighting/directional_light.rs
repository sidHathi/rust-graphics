use std::sync::Arc;

use cgmath::{
  ortho, Matrix4, Point3, SquareMatrix, Vector3
};
use futures::lock::Mutex;
use tokio::runtime::Runtime;
use wgpu::util::DeviceExt;

use crate::{graphics::{get_light_bind_group_info, get_light_buffer, get_render_pipeline, InstanceRaw, LightUniform, ModelVertex, Texture, Vertex}, util::log_buffer_data};

use super::{light_shadow_uniform::LightShadowUniform, shadow_map_pipeline::get_shadow_pipeline};

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
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("Copy buffer encoder")
    });
    let buffer_size = (config.width * config.height * std::mem::size_of::<f32>() as u32) as u64;

    // Create a buffer to copy the texture data to
    let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Readback Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Specify the texture copy operation
    encoder.copy_texture_to_buffer(
      wgpu::ImageCopyTexture {
        texture: &self.shadow_dt.texture,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
      },
      wgpu::ImageCopyBuffer {
        buffer: &readback_buffer,
        layout: wgpu::ImageDataLayout {
          offset: 0,
          bytes_per_row: Some(config.width * std::mem::size_of::<f32>() as u32),
          rows_per_image: Some(config.height as u32),
        },
      },
      wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
      },
    );

    queue.submit(std::iter::once(encoder.finish()));

    let buffer_slice: wgpu::BufferSlice = readback_buffer.slice(..);
    let (tx, rx) = futures::channel::oneshot::channel();

    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
      let _ = tx.send(result);
    });

    device.poll(wgpu::Maintain::Wait);

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
      rx
        .await
        .expect("communication failed")
        .expect("buffer reading failed");
      let slice: &[u8] = &buffer_slice.get_mapped_range();
      let mut num_shadowed: u32 = 0;
      let mut num_lit: u32 = 0;
      for (idx, val) in slice.iter().enumerate() {
        if *val != 0 {
          num_lit += 1;
          // println!("Val {} found at row {}, col {}", *val, idx/(config.width as usize), idx - (config.width as usize * (idx/(config.width as usize))));
        } else {
          num_shadowed += 1;
          // println!("Val {} found at row {}, col {}", *val, idx/(config.width as usize), idx - (config.width as usize * (idx/(config.width as usize))));
        }
      }
      // println!("Percentage of shadowed area: {} -> num lit pixels: {}, num shadowed: {}", (num_shadowed as f32)/((num_lit + num_shadowed) as f32), num_lit, num_shadowed);
      let _ = log_buffer_data(slice, config, "shadow_map_dump.log");
    });
  }
}