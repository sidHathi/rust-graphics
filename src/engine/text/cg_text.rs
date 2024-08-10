use std::result;

use cgmath::Vector4;
use wgpu::Color;

use crate::engine::{errors::EngineError, transforms::ModelTransform, Scene};

use super::font::{FontFace};

#[derive(Clone, Debug)]
pub struct CGText {
  pub transform: ModelTransform,
  pub font_size: f32,
  pub color: [f32; 4],
  pub opacity: f32,
  pub text: String,
  pub font_face: FontFace
}

impl From<&str> for CGText {
  fn from(content: &str) -> Self {
    Self {
      text: content.into(),
      font_face: FontFace::Default,
      transform: ModelTransform::default(),
      color: [0., 0., 0., 1.],
      opacity: 1.,
      font_size: 12.
    }
  }
}

impl CGText {
  pub fn new(content: &str) -> Self {
    Self {
      text: content.into(),
      font_face: FontFace::Default,
      transform: ModelTransform::default(),
      color: [0., 0., 0., 1.],
      opacity: 1.,
      font_size: 12.
    }
  }

  pub fn text(&self, new_text: &str) -> Self {
    let mut new = self.clone();
    new.text = new_text.into();
    new
  }

  pub fn font(&self, font_face: FontFace) -> Self {
    let mut new = self.clone();
    new.font_face = font_face;
    return new
  }

  pub fn transform(&self, transform: ModelTransform) -> Self {
    let mut new = self.clone();
    new.transform = transform;
    return new
  }

  pub fn color(&self, color: [f32; 4]) -> Self {
    let mut new = self.clone();
    new.color = color;
    return new
  }

  pub fn opacity(&self, opacity: f32) -> Self {
    let mut new = self.clone();
    new.opacity = opacity;
    return new
  }

  pub fn font_size(&self, font_size: f32) -> Self {
    let mut new = self.clone();
    new.font_size = font_size;
    return new
  }
  
  pub fn get_glyph_text(&self) -> glyph_brush::Text {
    glyph_brush::Text::new(&self.text)
      .with_color(self.color)
      .with_scale(self.font_size)
  }

  pub fn render(&self, scene: &mut Scene) {
    scene.draw_text(self);
  }
}