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

use state::State;
pub use model::{
  Mesh,
  Material,
  ModelVertex
};

pub async fn run() {
  env_logger::init();
  let event_loop = EventLoop::new();
  let window = WindowBuilder::new().build(&event_loop).unwrap();

  let mut state = State::new(window).await;
  let mut last_render_time = instant::Instant::now();

  event_loop.run(move |event, _, control_flow| match event {
    Event::DeviceEvent {
      event: DeviceEvent::MouseMotion{ delta, },
      .. // We're not using device_id currently
    } => if state.mouse_pressed {
      state.camera_controller.process_mouse(delta.0, delta.1)
    },
    Event::WindowEvent {
      ref event,
      window_id,
    } if window_id == state.window().id() => if !state.input(event) {
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
          state.resize(*physical_size);
        },
        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
          state.resize(**new_inner_size)
        },
        _ => {}
      }
    },
    Event::RedrawRequested(window_id) if window_id == state.window().id() => {
      let now = instant::Instant::now();
      let dt = now - last_render_time;
      last_render_time = now;
      state.update(dt);
      match state.render() {
        Ok(_) => {}
        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
        Err(e) => eprintln!("{:?}", e),
      }
    },
    Event::MainEventsCleared => {
      state.window().request_redraw();
    }
    _ => {}
  });
}
