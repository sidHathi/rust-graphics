use cgmath::{ortho, perspective, InnerSpace, Matrix4, Point3, Rad, Vector3};

use crate::graphics::OPENGL_TO_WGPU_MATRIX;

use super::DirectionalLight;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightShadowUniform {
  pub view_pos: [f32; 4],
  pub view_proj: [[f32; 4]; 4],
  pub direction: [f32; 4],
  pub color: [f32; 4],
}

impl LightShadowUniform {
  pub fn new_perspective(
    pos: Point3<f32>,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
    aspect: f32,
    pitch: Rad<f32>,
    yaw: Rad<f32>,
    color: Vector3<f32>,
  ) -> Self {
    let proj_matrix = OPENGL_TO_WGPU_MATRIX * perspective(fovy, aspect, znear, zfar);

    let (sin_pitch, cos_pitch) = pitch.0.sin_cos();
    let (sin_yaw, cos_yaw) = yaw.0.sin_cos();
    let view_matrix = Matrix4::look_to_rh(
      pos,
      Vector3::new(
          cos_pitch * cos_yaw,
          sin_pitch,
          cos_pitch * sin_yaw
      ).normalize(),
      Vector3::unit_y(),
    );
    let view_proj_matrix = proj_matrix * view_matrix;
    let view_pos_vec = pos.to_homogeneous();

    Self {
      view_pos: view_pos_vec.into(),
      view_proj: view_proj_matrix.into(),
      direction: view_pos_vec.into(),
      color: color.extend(1.).into()
    }
  }

  pub fn new_directional(
    pos: Point3<f32>,
    dir: Vector3<f32>,
    dims: Vector3<f32>,
    color: Vector3<f32>,
  ) -> Self {
    let view = Matrix4::look_to_rh(
      pos,
      dir, 
      Vector3::unit_y()
    );
    let proj = ortho(-dims.x, dims.x, -dims.y, dims.y, -dims.z, dims.z);
    Self {
      view_pos: pos.to_homogeneous().into(),
      view_proj: (proj * view).into(),
      direction: dir.extend(1.).into(),
      color: color.extend(1.).into()
    }
  }

  pub fn update_directional_view_proj(&mut self, light: &DirectionalLight) {
    self.view_pos = light.pos.to_homogeneous().into();
    self.view_proj = light.view_proj().into();
    self.direction = light.dir.extend(1.).into()
  }
}