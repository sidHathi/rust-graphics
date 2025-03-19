use cgmath::{ortho, perspective, Angle, InnerSpace, Matrix4, Point3, Rad, Vector3};

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

pub struct LightShadowUniformConstructionProps {
  pub pos: Point3<f32>,
  pub fovy: Rad<f32>,
  pub znear: f32,
  pub zfar: f32,
  pub aspect: f32,  
  pub pitch: Rad<f32>,
  pub yaw: Rad<f32>,
  pub color: Vector3<f32>,
}

impl LightShadowUniform {
  pub fn new_perspective(construction_props: LightShadowUniformConstructionProps) -> Self {
    let LightShadowUniformConstructionProps {
      pos,
      fovy,
      znear,
      zfar,
      aspect,   
      pitch,
      yaw,
      color,
    } = construction_props;
    // Correctly calculate direction vector from pitch and yaw
    let direction = Vector3::new(
      yaw.cos() * pitch.cos(),
      pitch.sin(),                
      yaw.sin() * pitch.cos()
    ).normalize();
    
    // Create target point that the light looks at
    let target = pos + direction;
    
    // Use right-handed view matrix with correct up vector
    // Try different up vectors if shadows are still flipped
    let up = Vector3::new(0.0, 1.0, 0.0);
    let view = Matrix4::look_at_rh(pos, target, up);
    
    // Create depth-corrected perspective matrix for WebGPU
    let proj = cgmath::perspective(fovy, aspect, znear, zfar);
    
    // Try this if shadows are flipped vertically:
    // let mut proj = cgmath::perspective(fovy, aspect, znear, zfar);
    // proj.y.y *= -1.0; // Flip y-axis
    Self {
      view_pos: pos.to_homogeneous().into(),
      view_proj: (proj * view).into(),
      direction: direction.extend(0.0).into(),
      color: color.extend(0.0).into(),
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