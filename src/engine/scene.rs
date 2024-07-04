use std::sync::{Arc, Mutex};

use cgmath::Rotation3;
use winit::{event::{ElementState, KeyboardInput, MouseButton, WindowEvent}, window::Window};
use wgpu::{util::DeviceExt};

use crate::graphics::{get_light_bind_group_info, get_light_buffer, get_render_pipeline, Camera, CameraController, CameraUniform, LightUniform, Projection, Texture};

use super::{component::Component, component_models::ComponentModels, test_component::TestComponent};

// initial goal -> render a single component with a model
// scene should essentially be akin to state from tutorial with a few additions
// i.e. it manages the overarching render and update for all child components
struct Scene {
  window: Window,
  size: winit::dpi::PhysicalSize<u32>,
  device: wgpu::Device,
  queue: wgpu:: Queue,
  config: wgpu::SurfaceConfiguration,
  surface: wgpu::Surface,
  components: Vec<Component<'static>>,
  projection: Projection,
  depth_texture: Texture,
  camera: Camera,
  camera_uniform: CameraUniform,
  camera_controller: CameraController,
  camera_buffer: wgpu::Buffer,
  light_uniform: LightUniform,
  light_buffer: wgpu::Buffer,
  light_bind_group_layout: wgpu::BindGroupLayout,
  light_bind_group: wgpu::BindGroup,
  light_render_pipeline: wgpu::RenderPipeline,
  mouse_pressed: bool,
  component_models: Arc<Mutex<ComponentModels>>,
}

impl Scene {
  pub async fn new(window: Window) -> Scene {
    // initialize components, camera, lights

    // wgpu setup
    let size = window.inner_size();

    let instance = wgpu::Instance::new(
      wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
      }
    );

    let surface = unsafe {
      instance.create_surface(&window)
    }.unwrap();

    let adapter = instance.request_adapter(
      &wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
      }
    ).await.unwrap();

    let (device, queue) = adapter.request_device(
      &wgpu::DeviceDescriptor {
        features: wgpu::Features::empty(),
        limits: if cfg!(target_arch = "wasm32") {
          wgpu::Limits::downlevel_webgl2_defaults()
        } else {
          wgpu::Limits::default()
        },
        label: None
      }, 
      None
    ).await.unwrap();

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
      view_formats: vec![]
    };
    surface.configure(&device, &config);

    //camera
    let camera = Camera::new(
      (0.0, 5.0, 10.0),
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

    // lighting
    let light_uniform = LightUniform {
      position: [2.0, 10.0, 2.0],
      _padding: 0,
      color: [1.0, 1.0, 1.0],
      _padding_2: 0,
    };
    let light_buffer = get_light_buffer(&device, &light_uniform);
    let (light_bind_group_layout, light_bind_group) = get_light_bind_group_info(&device, &light_buffer);

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
        source: wgpu::ShaderSource::Wgsl(include_str!("../graphics/light.wgsl").into()),
      };

      use crate::graphics::{
        ModelVertex,
        Vertex
      };
      get_render_pipeline(
        &device, 
        &layout, 
        config.format, 
        Some(Texture::DEPTH_FORMAT),
        &[ModelVertex::desc()],
        shader,
        "vs_main", 
        "fs_main"
      )
    };

    // texture bind group
    let texture_bind_group_layout = device.create_bind_group_layout(
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
          wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
              multisampled: false,
              sample_type: wgpu::TextureSampleType::Float { filterable: true },
              view_dimension: wgpu::TextureViewDimension::D2
            },
            count: None
          },
          wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None
          }
        ] 
      }
    );


    // load a depth texture
    let depth_texture = Texture::create_depth_texture(&device, &&config, "depth texture");

    // model store
    let model_store = Arc::new(Mutex::new(ComponentModels::new()));
    let underlying = TestComponent::new();
    let mut component = Component::new(underlying);
    component.init(None, model_store.clone(), &device, &queue, &texture_bind_group_layout);

    let mut components: Vec<Component> = Vec::new();
    components.push(component);

    Self {
      window,
      size,
      device,
      queue,
      config,
      surface,
      component_models: model_store,
      components,
      projection,
      depth_texture,
      camera,
      camera_uniform,
      camera_controller,
      light_uniform,
      light_buffer,
      light_bind_group_layout,
      light_bind_group,
      camera_buffer,
      light_render_pipeline,
      mouse_pressed: false,
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

  pub fn input (&mut self, event: &WindowEvent) -> bool {
    match event {
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
    // should also call component updates

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
    Ok(())
  }
}