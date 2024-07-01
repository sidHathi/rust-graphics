use cgmath::{InnerSpace, Point3};
use wgpu::util::DeviceExt;
use winit::{event::{ElementState, KeyboardInput, MouseButton, WindowEvent}, window::Window};
use cgmath::prelude::*;
use crate::debug::{
  self, DebugCubeNet, DrawDebugNet
};

use crate::{graphics::{model::{self, Vertex}, pipeline::get_render_pipeline, Texture}, sdf::{DrawIVModel, InferredVertexModel, SdfBounds, SdfShape, Shape}, util::Point};

use super::{camera::{Camera, CameraController, CameraUniform, Projection}, lighting};

pub struct IVState {
  pub surface: wgpu::Surface,
  pub device: wgpu::Device,
  pub queue: wgpu:: Queue,
  pub config: wgpu::SurfaceConfiguration,
  pub size: winit::dpi::PhysicalSize<u32>,
  pub window: Window,
  pub render_pipeline: wgpu::RenderPipeline,
  pub camera: Camera,
  pub projection: Projection,
  pub camera_uniform: CameraUniform,
  pub camera_controller: CameraController,
  pub camera_bind_group: wgpu::BindGroup,
  pub camera_buffer: wgpu::Buffer,
  pub depth_texture: Texture,
  pub iv_model: InferredVertexModel,
  pub debug_net: DebugCubeNet,
  pub debug_render_pipeline: wgpu::RenderPipeline,
  pub light_uniform: lighting::LightUniform,
  pub light_buffer: wgpu::Buffer,
  pub light_bind_group_layout: wgpu::BindGroupLayout,
  pub light_bind_group: wgpu::BindGroup,
  pub light_render_pipeline: wgpu::RenderPipeline,
  pub mouse_pressed: bool,
  clear_color: (f64, f64, f64, f64)
}

impl IVState {
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


    // camera setup
    let camera = Camera::new(
      (0.0, 30.0, 40.0),
      cgmath::Deg(-90.0), 
      cgmath::Deg(-20.0),
    );
    let projection = Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);
    let camera_controller = CameraController::new(4.0, 0.4);

    let mut camera_uniform = CameraUniform::new();
    camera_uniform.update_view_proj(&camera, &projection);

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

    let depth_texture = Texture::create_depth_texture(&device, &config, "depth texture");

    // lighting
    let light_uniform = lighting::LightUniform {
      position: [2.0, 10.0, 2.0],
      _padding: 0,
      color: [1.0, 1.0, 1.0],
      _padding_2: 0,
    };
    let light_buffer = lighting::get_light_buffer(&device, &light_uniform);
    let (light_bind_group_layout, light_bind_group) = lighting::get_light_bind_group_info(&device, &light_buffer);

    let light_render_pipeline = {
      let layout = device.create_pipeline_layout(
        &wgpu::PipelineLayoutDescriptor {
          label: Some("light pipeline layout"),
          bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
          push_constant_ranges: &[],
        }
      );

      let shader = wgpu::ShaderModuleDescriptor {
        label: Some("light shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("light.wgsl").into()),
      };

      use model::Vertex;
      get_render_pipeline(
        &device, 
        &layout, 
        config.format, 
        Some(Texture::DEPTH_FORMAT),
        &[model::ModelVertex::desc()],
        shader,
        "vs_main", 
        "fs_main"
      )
    };

    let sdf: SdfShape = SdfShape::new(Shape::Sphere { 
      center: Point3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
      }, 
      rad: 0.1 }, 
      |shape, point| {
        match shape {
          Shape::Sphere { center, rad } => {
            (point - center).magnitude() - rad
          },
          _ => 0.0
        }
      }
    );

    let bounds = SdfBounds {
      xmin: -0.21,
      xmax: 0.21,
      ymin: -0.21,
      ymax: 0.21,
      zmin: -0.21,
      zmax: 0.21
    };

    let iv_model = InferredVertexModel::new(&device, &queue, sdf, bounds, 0.05, &[200, 100, 0, 255]);

    // draw debug cubes
    let debug_net = DebugCubeNet::new(&device, &config, iv_model.vertex_coords.clone(), 0.1);
    
    // regular render pipeline
    let clear_color = (0.1, 0.2, 0.3, 1.0);

    let shader = wgpu::ShaderModuleDescriptor {
      label: Some("shader"),
      source: wgpu::ShaderSource::Wgsl(include_str!("iv-shader.wgsl").into())
    };

    let render_pipeline_layout = device.create_pipeline_layout(
      &wgpu::PipelineLayoutDescriptor {
        label: Some("Render pipeline layout"),
        bind_group_layouts: &[
          &iv_model.diffuse_bind_group_layout,
          &camera_bind_group_layout,
          &light_bind_group_layout,
        ],
        push_constant_ranges: &[]
      }
    );

    let render_pipeline = get_render_pipeline(
      &device, 
      &render_pipeline_layout, 
      config.format, 
      Some(Texture::DEPTH_FORMAT), 
      &[
        model::ModelVertex::desc()
      ], 
      shader, 
      "vs_main", 
      "fs_main"
    );


    // need debug render pipeline here
    let debug_shader = wgpu::ShaderModuleDescriptor {
      label: Some("debug shader"),
      source: wgpu::ShaderSource::Wgsl(include_str!("../debug/debug-shader.wgsl").into())
    };

    let debug_render_pipeline_layout = device.create_pipeline_layout(
      &wgpu::PipelineLayoutDescriptor {
        label: Some("Debug pipeline layout"),
        bind_group_layouts: &[
          &camera_bind_group_layout,
          &light_bind_group_layout,
        ],
        push_constant_ranges: &[]
      }
    );

    let debug_render_pipeline = get_render_pipeline(
      &device, 
      &debug_render_pipeline_layout, 
      config.format, 
      Some(Texture::DEPTH_FORMAT), 
      &[
        debug::DebugVertex::desc(),
        debug::DebugInstanceRaw::desc()
      ],
      debug_shader, 
      "vs_main", 
      "fs_main"
    );

    Self {
      surface,
      device,
      window,
      queue,
      config,
      size,
      render_pipeline,
      camera,
      projection,
      camera_uniform,
      camera_controller,
      camera_bind_group,
      camera_buffer,
      depth_texture,
      iv_model,
      debug_net,
      debug_render_pipeline,
      light_uniform,
      light_buffer,
      light_bind_group_layout,
      light_bind_group,
      light_render_pipeline,
      mouse_pressed: false,
      clear_color
    }
  }

  pub fn window(&self) -> &Window {
    &self.window
  }

  pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.projection.resize(new_size.width, new_size.height);
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
      WindowEvent::KeyboardInput {
        input: 
          KeyboardInput {
              virtual_keycode: Some(key),
              state,
              ..
            },
        ..
      } => self.camera_controller.process_keyboard(*key, *state),
      WindowEvent::MouseWheel { delta, .. } => {
        self.camera_controller.process_scroll(delta);
        true
      }
      WindowEvent::MouseInput {
        button: MouseButton::Left,
        state,
        ..
      } => {
        self.mouse_pressed = *state == ElementState::Pressed;
        true
      }
      _ => false,
    }
  }

  pub fn update(&mut self, dt: instant::Duration) {
    self.camera_controller.update_camera(&mut self.camera, dt);
    self.camera_uniform.update_view_proj(&self.camera, &self.projection);
    self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));

    let old_light_position: cgmath::Vector3<_> = self.light_uniform.position.into();
    self.light_uniform.position = 
    (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(60.0 * dt.as_secs_f32()))
        * old_light_position)
        .into();
    self.queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light_uniform]));
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

      use super::model::DrawLight;
      render_pass.set_pipeline(&self.light_render_pipeline);
      // render_pass.draw_light_model(&self.obj_model, &self.camera_bind_group, &self.light_bind_group);

      // use super::super::sdf::DrawIVModel;
      // render_pass.set_pipeline(&self.render_pipeline);
      // render_pass.draw_iv_model(&self.iv_model, &self.camera_bind_group, &self.light_bind_group);

      use crate::debug::DrawDebugNet;
      render_pass.set_pipeline(&self.debug_render_pipeline);
      render_pass.draw_debug_net(&self.debug_net, &self.camera_bind_group, &self.light_bind_group);
    }

    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
  }
}
