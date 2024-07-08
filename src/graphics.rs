use winit::{
  event::*,
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder,
};

mod state;
mod pipeline;
mod vertex;
mod texture;
mod camera;
mod instance;
mod model;
mod resources;
mod lighting;
mod iv_state;

use state::State;
pub use model::{
  Mesh,
  Material,
  ModelVertex,
  Vertex,
  Model,
  DrawLight,
  DrawModel,
};
pub use instance::{
  Instance,
  InstanceRaw
};
pub use resources::*;
pub use texture::Texture;
pub use pipeline::get_render_pipeline;
pub use camera::{
  Camera,
  CameraController,
  Projection,
  CameraUniform
};
pub use lighting::*;

use self::iv_state::IVState;
use super::playground::pg_state::PgState;
use super::engine::Scene;

pub async fn run() {
  env_logger::init();
  let event_loop = EventLoop::new();
  let window = WindowBuilder::new().build(&event_loop).unwrap();

  // let mut state = State::new(window).await;
  // let mut iv_state: IVState = IVState::new(window).await;
  // let mut pg_state: PgState = PgState::new(window).await;
  let mut scene: Scene = Scene::new(window).await;
  let mut last_render_time = instant::Instant::now();

  event_loop.run(move |event, _, control_flow| match event {
    Event::DeviceEvent {
      event: DeviceEvent::MouseMotion{ delta, },
      .. // We're not using device_id currently
    } => if scene.mouse_pressed {
      scene.camera_controller.process_mouse(delta.0, delta.1)
    },
    Event::WindowEvent {
      ref event,
      window_id,
    } if window_id == scene.window().id() => if !scene.input(event) {
      match event {
        WindowEvent::CloseRequested
        | WindowEvent::KeyboardInput {
          input:
            KeyboardInput {
              state: ElementState::Pressed,
              virtual_keycode: Some(VirtualKeyCode::Escape),
              ..
            },
          ..
        } => *control_flow = ControlFlow::Exit,
        WindowEvent::Resized(physical_size) => {
          scene.resize(*physical_size);
        },
        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
          scene.resize(**new_inner_size)
        },
        _ => {}
      }
    },
    Event::RedrawRequested(window_id) if window_id == scene.window().id() => {
      let now = instant::Instant::now();
      let dt = now - last_render_time;
      last_render_time = now;
      scene.update(dt);
      match scene.render() {
        Ok(_) => {}
        Err(wgpu::SurfaceError::Lost) => scene.resize(scene.size),
        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
        Err(e) => eprintln!("{:?}", e),
      }
    },
    Event::MainEventsCleared => {
      scene.window().request_redraw();
    }
    _ => {}
  });
}
