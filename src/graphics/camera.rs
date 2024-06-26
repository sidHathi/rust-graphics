use cgmath::{
  SquareMatrix,
  Point3,
  Rad,
  Matrix4,
  Vector3,
  InnerSpace,
  perspective,
};
use winit::event::*;
use winit::dpi::PhysicalPosition;
use instant::Duration;
use std::f32::consts::FRAC_PI_2;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct Camera {
  pub position: Point3<f32>,
  pub yaw: Rad<f32>,
  pub pitch: Rad<f32>,
}

impl Camera {
  // pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
  //   let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
  //   let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
  //   return OPENGL_TO_WGPU_MATRIX * proj * view;
  // }
  pub fn new<
    V: Into<Point3<f32>>,
    Y: Into<Rad<f32>>,
    P: Into<Rad<f32>>
  >(
    position: V,
    yaw: Y,
    pitch: P
  ) -> Self {
    Self {
      position: position.into(),
      yaw: yaw.into(),
      pitch: pitch.into()
    }
  }

  pub fn calc_matrix(&self) -> Matrix4<f32> {
    let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
    let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

    Matrix4::look_to_rh(
      self.position,
      Vector3::new(
          cos_pitch * cos_yaw,
          sin_pitch,
          cos_pitch * sin_yaw
      ).normalize(),
      Vector3::unit_y(),
    )
  }
}

pub struct Projection {
  aspect: f32,
  fovy: Rad<f32>,
  znear: f32,
  zfar: f32,
}

impl Projection {
  pub fn new<F: Into<Rad<f32>>>(
    width: u32,
    height: u32,
    fovy: F,
    znear: f32,
    zfar: f32,
  ) -> Self {
    Self {
      aspect: width as f32 / height as f32,
      fovy: fovy.into(),
      znear,
      zfar,
    }
  }

  pub fn resize(&mut self, width: u32, height: u32) {
    self.aspect = width as f32 / height as f32;
  }

  pub fn calc_matrix(&self) -> Matrix4<f32> {
    OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
  }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
  pub view_pos: [f32; 4],
  pub view_proj: [[f32; 4]; 4]
}


impl CameraUniform {
  pub fn new() -> Self {
    Self {
      view_pos: [0.0; 4],
      view_proj: cgmath::Matrix4::identity().into()
    }
  }

  pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
    self.view_pos = camera.position.to_homogeneous().into();
    self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
  }
}

pub struct CameraController {
  pub amount_left: f32,
  pub amount_right: f32,
  pub amount_forward: f32,
  pub amount_backward: f32,
  pub amount_up: f32,
  pub amount_down: f32,
  pub rotate_horizontal: f32,
  pub rotate_vertical: f32,
  pub scroll: f32,
  pub speed: f32,
  pub sensitivity: f32,
}

impl CameraController {
  pub fn new(speed: f32, sensitivity: f32) -> Self {
    Self {
      amount_left: 0.0,
      amount_right: 0.0,
      amount_forward: 0.0,
      amount_backward: 0.0,
      amount_up: 0.0,
      amount_down: 0.0,
      rotate_horizontal: 0.0,
      rotate_vertical: 0.0,
      scroll: 0.0,
      speed,
      sensitivity,
    }
  }

  pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool{
    let amount = if state == ElementState::Pressed { 1.0 } else { 0.0 };
    match key {
      VirtualKeyCode::W | VirtualKeyCode::Up => {
        self.amount_forward = amount;
        true
      }
      VirtualKeyCode::S | VirtualKeyCode::Down => {
        self.amount_backward = amount;
        true
      }
      VirtualKeyCode::A | VirtualKeyCode::Left => {
        self.amount_left = amount;
        true
      }
      VirtualKeyCode::D | VirtualKeyCode::Right => {
        self.amount_right = amount;
        true
      }
      VirtualKeyCode::Space => {
        self.amount_up = amount;
        true
      }
      VirtualKeyCode::LShift => {
        self.amount_down = amount;
        true
      }
      _ => false,
    }
  }

  pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
    self.rotate_horizontal = mouse_dx as f32;
    self.rotate_vertical = mouse_dy as f32;
  }

  pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
    self.scroll = -match delta {
      // I'm assuming a line is about 100 pixels
      MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
      MouseScrollDelta::PixelDelta(PhysicalPosition {
          y: scroll,
          ..
      }) => *scroll as f32,
    };
  }

  pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
    let dt = dt.as_secs_f32();

    // Move forward/backward and left/right
    let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
    let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
    let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
    camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
    camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

    // Move in/out (aka. "zoom")
    // Note: this isn't an actual zoom. The camera's position
    // changes when zooming. I've added this to make it easier
    // to get closer to an object you want to focus on.
    let (pitch_sin, pitch_cos) = camera.pitch.0.sin_cos();
    let scrollward = Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
    camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
    self.scroll = 0.0;

    // Move up/down. Since we don't use roll, we can just
    // modify the y coordinate directly.
    camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

    // Rotate
    camera.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
    camera.pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;

    // If process_mouse isn't called every frame, these values
    // will not get set to zero, and the camera will rotate
    // when moving in a non-cardinal direction.
    self.rotate_horizontal = 0.0;
    self.rotate_vertical = 0.0;

    // Keep the camera's angle from going too high/low.
    if camera.pitch < -Rad(SAFE_FRAC_PI_2) {
      camera.pitch = -Rad(SAFE_FRAC_PI_2);
    } else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
      camera.pitch = Rad(SAFE_FRAC_PI_2);
    }
  }
}