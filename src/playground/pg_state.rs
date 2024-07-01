use crate::{graphics::{get_render_pipeline, ModelVertex, Texture}, playground::{pg_triangle::DrawPgTriangle, pg_vertex}};

use super::{pg_cube::{self, PgCube}, pg_triangle::{self, PgTriangle}};
use cgmath::{Point3, Vector3};
use wgpu::util::DeviceExt;
use winit::{event::WindowEvent, window::Window};

pub struct PgState {
  pub surface: wgpu::Surface,
  pub device: wgpu::Device,
  pub window: Window,
  pub config: wgpu::SurfaceConfiguration,
  pub size: winit::dpi::PhysicalSize<u32>,
  pub queue: wgpu::Queue,
  pub clear_color: (f64, f64, f64, f64),
  pub depth_texture: Texture,
  pub pg_cube: PgCube,
  pub pg_triangle: PgTriangle,
  pub render_pipeline_layout: wgpu::PipelineLayout,
  pub render_pipeline: wgpu::RenderPipeline
}

impl PgState {
  pub async fn new(window: Window) -> PgState {
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

    let pg_cube = PgCube::new(&device, Point3 { x: 0., y: 0., z: 0.5 }, 0.5);
    let pg_triangle = PgTriangle::new(&device, Point3 { x: 0., y: 0., z: 1. }, 1.);

    let shader = wgpu::ShaderModuleDescriptor {
      label: Some("Playground shader"),
      source: wgpu::ShaderSource::Wgsl(include_str!("pg_shader.wgsl").into()),
    };

    let render_pipeline_layout = device.create_pipeline_layout(
      &wgpu::PipelineLayoutDescriptor {
        label: Some("PG pipeline layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
      }
    );

    let render_pipeline = get_render_pipeline(
      &device, 
      &render_pipeline_layout, 
      config.format, 
      Some(Texture::DEPTH_FORMAT), 
      &[
        pg_vertex::PgVertex::desc()
      ], 
      shader, 
      "vs_main", 
      "fs_main"
    );

    let depth_texture = Texture::create_depth_texture(&device, &config, "depth texture");

    Self {
      surface,
      device,
      window,
      size,
      queue,
      config,
      pg_cube,
      pg_triangle,
      depth_texture,
      render_pipeline_layout,
      render_pipeline,
      clear_color: (0.1, 0.2, 0.3, 1.)
    }
  }

  pub fn window(&self) -> &Window {
    &self.window
  }
  
  pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    // if new_size.width > 0 && new_size.height > 0 {
    //   self.projection.resize(new_size.width, new_size.height);
    //   self.size = new_size;
    //   self.config.width = new_size.width;
    //   self.config.height = new_size.height;
    //   self.surface.configure(&self.device, &self.config);
    //   self.depth_texture = Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
    // }

    if new_size.width > 0 && new_size.height > 0 {
      self.size = new_size;
      self.config.width = new_size.width;
      self.config.height = new_size.height;
      self.surface.configure(&self.device, &self.config);
    }
  }


  pub fn input(&mut self, event: &WindowEvent) -> bool {
    false
  }

  pub fn update(&mut self, dt: instant::Duration) {

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

      use pg_cube::DrawPgCube;
      render_pass.set_pipeline(&self.render_pipeline);
      render_pass.draw_cube(&self.pg_cube);
      // render_pass.draw_triangle(&self.pg_triangle);
    }

    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
  }
}