use std::{collections::HashMap, fmt::Debug, sync::{Arc, Mutex}};

use cgmath::{Rotation3, Vector2};
use winit::{event::{ElementState, KeyboardInput, MouseButton, WindowEvent}, window::Window};
use wgpu::{util::DeviceExt, BindGroupLayout};

use crate::graphics::{get_light_bind_group_info, get_light_buffer, get_render_pipeline, Camera, CameraController, CameraUniform, DrawModel, Instance, InstanceRaw, LightUniform, Model, Projection, Texture};

use super::{collisions::CollisionManager, component::Component, component_store::{ComponentKey, ComponentStore}, debug::{DebugRenderPipelineType, DebugRenderer}, errors::EngineError, events::{Event, EventManager}, model_renderer::ModelRenderer, mouse::Mouse, raycasting::RaycastManager, renderable_model::{RenderSettings, RenderableModel}, state::{create_app_state, Store}, test_component::TestComponent, transforms::ModelTransform};

// The Scene struct contains the data needed to render the wgpu scene
// It manages the camera, lighting and i/o. It also handles the operation
// of any and all Components within the scene
pub struct Scene {
  window: Window,
  pub size: winit::dpi::PhysicalSize<u32>,
  device: wgpu::Device,
  queue: wgpu:: Queue,
  config: wgpu::SurfaceConfiguration,
  surface: wgpu::Surface,
  pub components: ComponentStore,
  projection: Projection,
  depth_texture: Texture,
  texture_bind_group_layout: BindGroupLayout,
  camera: Camera,
  camera_uniform: CameraUniform,
  pub camera_controller: CameraController,
  camera_buffer: wgpu::Buffer,
  camera_bind_group: wgpu::BindGroup,
  light_uniform: LightUniform,
  light_buffer: wgpu::Buffer,
  light_bind_group_layout: wgpu::BindGroupLayout,
  light_bind_group: wgpu::BindGroup,
  light_render_pipeline: wgpu::RenderPipeline,
  pub mouse_pressed: bool,
  clear_color: (f64, f64, f64, f64),
  pub model_renderer: ModelRenderer,
  render_pipeline_layout: wgpu::PipelineLayout,
  render_pipeline: wgpu::RenderPipeline,
  pub app: Option<Component>, // top level component
  pub app_state: Store, // state manager
  pub event_manager: EventManager, // event manager
  pub collision_manager: CollisionManager, // collision manager
  pub raycast_manager: RaycastManager,
  pub mouse: Mouse,
  pub debug_renderer: DebugRenderer,
  pub debug_render_pipelines: HashMap<DebugRenderPipelineType, wgpu::RenderPipeline>,
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
      .unwrap_or(wgpu::TextureFormat::Rgba8Unorm);

    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: size.width,
      height: size.height,
      present_mode: surface_caps.present_modes[0],
      alpha_mode: wgpu::CompositeAlphaMode::PostMultiplied,
      view_formats: vec![]
    };
    surface.configure(&device, &config);

    //camera
    let camera = Camera::new(
      (0.0, 20.0, 50.0),
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
      position: [2.0, 200.0, 2.0],
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

    // render pipeline
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Render Pipeline Layout"),
      bind_group_layouts: &[
        &texture_bind_group_layout,
        &camera_bind_group_layout,
        &light_bind_group_layout,
      ],
      push_constant_ranges: &[],
    });
    
    use crate::graphics::{
      Vertex,
      ModelVertex,
      
    };
    // pipline init/config
    let render_pipeline = {
      let shader = wgpu::ShaderModuleDescriptor {
          label: Some("Normal Shader"),
          source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
      };
      get_render_pipeline(
        &device,
        &render_pipeline_layout,
        config.format,
        Some(Texture::DEPTH_FORMAT),
        &[ModelVertex::desc(), InstanceRaw::desc()],
        shader,
        "vs_main", 
        "fs_main"
      )
    };

    // model store, component store, state, events, collisions, initialized here
    let model_renderer = ModelRenderer::new();
    let mut components = ComponentStore::new();
    let app_state = create_app_state();
    let event_manager = EventManager::new();
    let collision_manager = CollisionManager::new();
    let raycast_manager = RaycastManager::new();
    let mouse = Mouse::new(10000.);
    let debug_renderer = DebugRenderer::new();
    let debug_render_pipelines = HashMap::new();

    let mut scene = Self {
      window,
      size,
      device,
      queue,
      config,
      surface,
      model_renderer,
      components,
      projection,
      depth_texture,
      texture_bind_group_layout,
      camera,
      camera_uniform,
      camera_controller,
      camera_bind_group,
      light_uniform,
      light_buffer,
      light_bind_group_layout,
      light_bind_group,
      camera_buffer,
      light_render_pipeline,
      render_pipeline,
      render_pipeline_layout,
      mouse_pressed: false,
      clear_color: (0.1, 0.2, 0.3, 1.),
      app: None,
      app_state,
      event_manager,
      collision_manager,
      raycast_manager,
      mouse,
      debug_renderer,
      debug_render_pipelines
    };

    println!("Scene initialized");
    // The main app component gets initialized here
    let underlying = TestComponent::new();
    let app = Component::new(
      underlying,
      &mut scene,
      None,
    ).await;
    scene.app = app;
    println!("App initialized");

    scene
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
      } => {
        self.event_manager.handle_event(Event::from(event).unwrap());
        self.camera_controller.process_keyboard(*key, *state)
      },
      WindowEvent::MouseWheel { delta, .. } => {
        self.camera_controller.process_scroll(delta);
        true
      },
      WindowEvent::MouseInput {
        button: MouseButton::Left,
        state,
        ..
      } => {
        self.mouse_pressed = *state == ElementState::Pressed;
        true
      },
      WindowEvent::CursorMoved { 
        position, 
        ..
      } => {
        self.mouse.update_mouse_state(Some(Vector2::new(position.x as f32, position.y as f32)), self.mouse_pressed, &self.camera, &self.projection, &self.config);
        true
      },
      WindowEvent::CursorLeft { .. } => {
        self.mouse.update_mouse_state(None, self.mouse_pressed, &self.camera, &self.projection, &self.config);
        true
      }
      _ => false,
    }
  }

  pub fn update(&mut self, dt: instant::Duration) {
    // trigger any event callbacks:
    self.event_manager.update(dt);
    self.app_state.update(dt);

    self.event_manager.trigger_callbacks(&mut self.components);
    let _ = self.app_state.trigger_callbacks(&mut self.components);

    let comp_clones: Vec<_> = self.components.iter().map(|(_, comp)| comp.clone()).collect();
    for comp in comp_clones.iter() {
      comp.update(self, dt);
    }

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
    // mark models to be rendered
    if let Some(app) = self.app.clone() {
      if let Err(err) = app.render(self, None) {
        println!("render failed with err {}", err);
      }
    } else {
      println!("No app found");
      return Ok(());
    }
    self.collision_manager.update_collider_positions(self.model_renderer.get_position_cache());
    self.raycast_manager.intersect_colliders(&self.collision_manager);
    self.mouse.intersect_colliders(&self.collision_manager);
    self.collision_manager.trigger_collision_events(&mut self.event_manager);
    self.raycast_manager.trigger_raycast_events(&mut self.event_manager);
    self.mouse.trigger_mouse_events(&mut self.event_manager);


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


      use crate::graphics::DrawLight;
      // render_pass.set_pipeline(&self.light_render_pipeline);
      // render_pass.draw_light_model(&self.obj_model, &self.camera_bind_group, &self.light_bind_group);

      render_pass.set_pipeline(&self.render_pipeline);
      for model_tuple in self.model_renderer.get_rendering_models() {
        // println!("Rendering model: {:?}, {:?}", &model_tuple.0, &model_tuple.1);
        render_pass.set_vertex_buffer(1, model_tuple.1.slice(..));
        render_pass.draw_model_instanced(&model_tuple.0, 0..1, &self.camera_bind_group, &self.light_bind_group);
      }

      use crate::engine::debug::DrawDebugRenderables;
      for (key, val) in self.debug_render_pipelines.iter() {
        render_pass.draw_debug_renderables(&self.debug_renderer, key.clone(), &val, &self.camera_bind_group);
      }
    }

    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();
    // clear model render list
    self.model_renderer.clear();
    self.debug_renderer.reset();
    Ok(())
  }

  pub async fn load_model(&mut self, filename: &str, instances: Option<Vec<Instance>>, component_key: ComponentKey) -> Result<RenderableModel, EngineError> {
    let load_res = self.model_renderer.load_model(filename, instances, component_key, &self.device, &self.queue, &self.texture_bind_group_layout).await;
    if let Ok(model) = load_res {
      return Ok(model)
    } else {
      println!("model load failed");
      return load_res;
    }
  }

  pub fn render_model(&mut self, model: &RenderableModel, render_settings: Option<RenderSettings>) -> Result<(), EngineError> {
    // needs to position/rotate the model appropriately too
    self.model_renderer.render(model, render_settings.unwrap_or(RenderSettings::default()), &self.queue, &self.device)
    // self.model_renderer.render_from_cache(model)
  }
}
