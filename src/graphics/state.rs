use wgpu::util::DeviceExt;
use winit::{
  dpi::PhysicalPosition, event::{ElementState, VirtualKeyCode, WindowEvent}, window::Window
};
use image::GenericImageView;
use cgmath::prelude::*;

use super::pipeline::get_render_pipeline;
use super::vertex::Vertex;
use super::texture::Texture;
use super::camera::{
  Camera,
  CameraController,
  CameraUniform,
};
use super::instance::{
  Instance,
  InstanceRaw
};
use super::model::{
  DrawModel,
  Model,
};
use super::resources::load_model;

const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, -0.25, -0.5], tex_coords: [0.0, 1.0] }, // A
    Vertex { position: [0.5, -0.25, -0.5], tex_coords: [1.0, 1.0] }, // B
    Vertex { position: [0., -0.25, 0.5], tex_coords: [0.5, 1.0] }, // C
    Vertex { position: [0., 0.25, 0.], tex_coords: [0.5, 0.0] }, // D
];

const INDICES: &[u16] = &[
  0, 1, 2,
  3, 2, 1,
  3, 0, 2,
  3, 1, 0,
];

const NUM_INSTANCES_PER_ROW: u32 = 10;
const SPACE_BETWEEN_INSTANCES: f32 = 30.0;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(NUM_INSTANCES_PER_ROW as f32 * 0.5, 0.0, NUM_INSTANCES_PER_ROW as f32 * 0.5);

pub struct State {
  pub surface: wgpu::Surface,
  pub device: wgpu::Device,
  pub queue: wgpu::Queue,
  pub config: wgpu::SurfaceConfiguration,
  pub size: winit::dpi::PhysicalSize<u32>,
  pub window: Window,
  pub render_pipeline: wgpu::RenderPipeline,
  pub vertex_buffer: wgpu::Buffer,
  pub index_buffer: wgpu::Buffer,
  pub num_vertices: u32,
  pub num_indices: u32,
  pub diffuse_bind_group_layout: wgpu::BindGroupLayout,
  pub diffuse_bind_group: wgpu::BindGroup,
  pub diffuse_texture: Texture,
  pub camera: Camera,
  pub camera_bind_group_layout: wgpu::BindGroupLayout,
  pub camera_bind_group: wgpu::BindGroup,
  pub camera_buffer: wgpu::Buffer,
  pub camera_uniform: CameraUniform,
  pub camera_controller: CameraController,
  pub instances: Vec<Instance>,
  pub instance_buffer: wgpu::Buffer,
  pub depth_texture: Texture,
  pub obj_model: Model,
  clear_color: (f64, f64, f64, f64),
  pos_shading: bool,
}

impl State {
  pub async fn new(window: Window) -> Self {
    let size = window.inner_size();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
      backends: wgpu::Backends::all(),
      ..Default::default()
    });

    let surface = unsafe {
      instance.create_surface(&window)
    }.unwrap();

    // adapter init
    let adapter = instance.request_adapter(
      &wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
      }
    ).await.unwrap();

    // device init
    let (device, queue) = adapter.request_device(
      &wgpu::DeviceDescriptor {
        features: wgpu::Features::empty(),
        limits: if cfg!(target_arch = "wasm32") {
          wgpu::Limits::downlevel_webgl2_defaults()
        } else {
          wgpu::Limits::default()
        },
        label: None,
      }, 
      None
    ).await.unwrap();

    // surface config
    let surface_caps = surface.get_capabilities(&adapter);

    let surface_format = surface_caps.formats.iter()
      .copied()
      .filter(|f| f.is_srgb())
      .next()
      .unwrap_or(surface_caps.formats[0]);
    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: size.width,
      height: size.height,
      present_mode: surface_caps.present_modes[0],
      alpha_mode: surface_caps.alpha_modes[0],
      view_formats: vec![],
    };
    surface.configure(&device, &config);

    // loading image texture
    let diffuse_bytes = include_bytes!("../stargate.jpeg");
    let diffuse_texture = Texture::from_bytes(&device, &queue, diffuse_bytes, "stargate.jpeg").unwrap();

    let diffuse_bind_group_layout = device.create_bind_group_layout(
      &wgpu::BindGroupLayoutDescriptor { 
        label: Some("Texture bind group layout"), 
        entries: &[
          wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
              multisampled: false,
              view_dimension: wgpu::TextureViewDimension::D2,
              sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
          },
          wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            // This should match the filterable field of the
            // corresponding Texture entry above.
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
          },
        ] 
      }
    );
    let diffuse_bind_group = device.create_bind_group(
      &wgpu::BindGroupDescriptor {
        layout: &diffuse_bind_group_layout,
        entries: &[
          wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
          },
          wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
          }
        ],
        label: Some("diffuse_bind_group"),
      }
    );

    // buffer creation
    let vertex_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
      }
    );
    let num_vertices = VERTICES.len() as u32;

    let index_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(INDICES),
        usage: wgpu::BufferUsages::INDEX
      }
    );
    let num_indices = INDICES.len() as u32;

    // camera setup
    let camera = Camera {
      eye: (0.0, 30.0, 40.0).into(),
      // have it look at the origin
      target: (0.0, 0.0, 0.0).into(),
      // which way is "up"
      up: cgmath::Vector3::unit_y(),
      aspect: config.width as f32 / config.height as f32,
      fovy: 45.0,
      znear: 0.1,
      zfar: 100.0,
    };

    let mut camera_uniform = CameraUniform::new();
    camera_uniform.update_view_proj(&camera);

    let camera_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("Camera Buffer"),
        contents: bytemuck::cast_slice(&[camera_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      }
    );

    let camera_bind_group_layout = device.create_bind_group_layout(
      &wgpu::BindGroupLayoutDescriptor {
        label: Some("camera_bind_group_layout"),
        entries : &[
          wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
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
    let camera_bind_group = device.create_bind_group(
      &wgpu::BindGroupDescriptor {
        layout: &camera_bind_group_layout,
        label: Some("camera_bind_group"),
        entries: &[
          wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_buffer.as_entire_binding(),
          }
        ]
      }
    );
    let camera_controller = CameraController::new(0.2);

    // instance setup
    let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
      (0..NUM_INSTANCES_PER_ROW).map(move |x| {
        let x = SPACE_BETWEEN_INSTANCES * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
        let z = SPACE_BETWEEN_INSTANCES * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

        let position =  cgmath::Vector3 { x: x as f32, y: 0.0, z: z as f32 };

        let rotation = if position.is_zero() {
          cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
        } else {
          cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
        };

        Instance {
          position, rotation
        }
      })
    }).collect::<Vec<_>>();

    let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
    let instance_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("Instance buffer"),
        contents: bytemuck::cast_slice(&instance_data),
        usage: wgpu::BufferUsages::VERTEX
      }
    );

    let depth_texture = Texture::create_depth_texture(&device, &&config, "depth texture");

    let obj_model = load_model("dice.obj", &device, &queue, &diffuse_bind_group_layout).await.unwrap();

    // pipline init/config
    let render_pipeline = get_render_pipeline(
      &device,
      &config, 
      &diffuse_bind_group_layout,
      &camera_bind_group_layout,
      "vs_main", 
      "fs_main"
    );

    // temp:
    let clear_color = (0.1, 0.2, 0.3, 1.0);

    Self {
      surface,
      device,
      queue,
      config,
      size,
      window,
      render_pipeline,
      vertex_buffer,
      index_buffer,
      num_vertices,
      num_indices,
      diffuse_bind_group,
      diffuse_bind_group_layout,
      diffuse_texture,
      camera,
      camera_bind_group_layout,
      camera_bind_group,
      camera_buffer,
      camera_uniform,
      camera_controller,
      instances,
      instance_buffer,
      depth_texture,
      obj_model,
      clear_color,
      pos_shading: false,
    }
  }

  pub fn window(&self) -> &Window {
    &self.window
  }

  pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.size = new_size;
      self.config.width = new_size.width;
      self.config.height = new_size.height;
      self.surface.configure(&self.device, &self.config);
      self.depth_texture = Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
    }
  }

  pub fn input(&mut self, event: &WindowEvent) -> bool {
    match event {
      WindowEvent::CursorMoved { position,.. } => {
        // println!("pos: x: {}, y: {}", position.x, position.y);
        self.clear_color.0 = position.x / self.window().inner_size().width as f64;
        self.clear_color.1 = position.y / self.window().inner_size().width as f64;
        true
      },
      WindowEvent::KeyboardInput { input, .. } => {
        if let Some(key) = input.virtual_keycode {
          if key == VirtualKeyCode::Space && input.state == ElementState::Pressed {
            if self.pos_shading {
              self.render_pipeline = get_render_pipeline(
                &self.device, 
                &self.config,
                &self.diffuse_bind_group_layout,
                &self.camera_bind_group_layout,
                "vs_main",
                "fs_main"
              );
              self.pos_shading = false;
            } else {
              self.render_pipeline = get_render_pipeline(
                &self.device, 
                &self.config,
                &self.diffuse_bind_group_layout,
                &self.camera_bind_group_layout,
                "vs_main",
                "fs_pos"
              );
              self.pos_shading = true;
            }
            return true;
          } else {
            return self.camera_controller.process_events(event);
          };
        }
        false
      }
      _ => false,
    }
  }

  pub fn update(&mut self) {
    self.camera_controller.update_camera(&mut self.camera);
    self.camera_uniform.update_view_proj(&self.camera);
    self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
  }

  pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    let output = self.surface.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("Render encoder")
    });

    {
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { 
        label: Some("Render pass"), 
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color {
              r: self.clear_color.0,
              g: self.clear_color.1,
              b: self.clear_color.2,
              a: self.clear_color.3,
            }),
            store: wgpu::StoreOp::Store,
          },
        })], 
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
          view: &self.depth_texture.view,
          depth_ops: Some(wgpu::Operations {
              load: wgpu::LoadOp::Clear(1.0),
              store: wgpu::StoreOp::Store,
          }),
          stencil_ops: None,
        }), 
        timestamp_writes: None, 
        occlusion_query_set: None 
      });

      render_pass.set_pipeline(&self.render_pipeline);
      // render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
      // render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
      // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
      render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
      // render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

      // render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as _);

      render_pass.draw_model_instanced(&self.obj_model, 0..self.instances.len() as _, &self.camera_bind_group);
    }

    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
  }
}