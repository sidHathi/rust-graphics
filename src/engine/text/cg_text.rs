


use glyph_brush::{BuiltInLineBreaker, Layout, Section};
use wgpu::Color;

use crate::engine::{transforms::ModelTransform, Scene};

use super::{font::FontFace};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum TextAlignment {
  Left,
  Center,
  Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum VerticalTextAlignment {
  Top,
  Center,
  Bottom,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum TextWrapStyle {
  NoWrap,
  Wrap,
}

#[derive(Clone, Debug)]
pub struct CGText {
  pub transform: ModelTransform,
  pub font_size: f32,
  pub color: Color,
  pub opacity: f32,
  pub text: String,
  pub wrap_style: TextWrapStyle,
  pub text_alignment: TextAlignment,
  pub vertical_text_alignment: VerticalTextAlignment,
  pub font_face: FontFace,
  pub max_width: Option<f32>,
  pub max_height: Option<f32>,
}

impl From<&str> for CGText {
  fn from(content: &str) -> Self {
    Self {
      text: content.into(),
      font_face: FontFace::Default,
      transform: ModelTransform::default(),
      color: Color::BLACK,
      opacity: 1.,
      font_size: 12.,
      wrap_style: TextWrapStyle::NoWrap,
      text_alignment: TextAlignment::Center,
      vertical_text_alignment: VerticalTextAlignment::Top,
      max_height: None,
      max_width: None,
    }
  }
}

impl CGText {
  pub fn new(content: &str) -> Self {
    Self {
      text: content.into(),
      font_face: FontFace::Default,
      transform: ModelTransform::default(),
      color: Color::BLACK,
      opacity: 1.,
      font_size: 12.,
      max_height: None,
      max_width: None,
      wrap_style: TextWrapStyle::NoWrap,
      text_alignment: TextAlignment::Center,
      vertical_text_alignment: VerticalTextAlignment::Top,
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
    new
  }

  pub fn transform(&self, transform: ModelTransform) -> Self {
    let mut new = self.clone();
    new.transform = transform;
    new
  }

  pub fn color(&self, new_color: Color) -> Self {
    let mut new = self.clone();
    new.color = new_color;
    new
  }

  pub fn opacity(&self, opacity: f32) -> Self {
    let mut new = self.clone();
    new.opacity = opacity;
    new
  }

  pub fn font_size(&self, font_size: f32) -> Self {
    let mut new = self.clone();
    new.font_size = font_size * 50.;
    new
  }

  pub fn wrap(&self, wrap: TextWrapStyle) -> Self {
    let mut new = self.clone();
    new.wrap_style = wrap;
    new
  }

  pub fn max_width(&self, max_width: f32) -> Self {
    let mut new = self.clone();
    new.max_width = Some(max_width);
    new
  }
  
  pub fn max_height(&self, max_height: f32) -> Self {
    let mut new = self.clone();
    new.max_height = Some(max_height);
    new
  }

  pub fn align(&self, new_alignment: TextAlignment) -> Self {
    let mut new = self.clone();
    new.text_alignment = new_alignment;
    new
  }

  pub fn align_vertical(&self, new_alignment: VerticalTextAlignment) -> Self {
    let mut new = self.clone();
    new.vertical_text_alignment = new_alignment;
    new
  }


  pub fn get_glyph_text(&self) -> glyph_brush::Text {
    glyph_brush::Text::new(&self.text)
      .with_scale(self.font_size)
  }

  pub fn get_glyph_section(&self) -> Section {
    let h_align = match self.text_alignment {
      TextAlignment::Left => glyph_brush_layout::HorizontalAlign::Left,
      TextAlignment::Center => glyph_brush_layout::HorizontalAlign::Center,
      TextAlignment::Right => glyph_brush_layout::HorizontalAlign::Right
    };
    let v_align = match self.vertical_text_alignment {
      VerticalTextAlignment::Top => glyph_brush_layout::VerticalAlign::Top,
      VerticalTextAlignment::Center => glyph_brush_layout::VerticalAlign::Center,
      VerticalTextAlignment::Bottom => glyph_brush_layout::VerticalAlign::Bottom
    };

    let layout = match self.wrap_style {
      TextWrapStyle::Wrap => Layout::default_wrap()
        .h_align(h_align)
        .v_align(v_align),
      TextWrapStyle::NoWrap => glyph_brush::Layout::SingleLine { 
        line_breaker: BuiltInLineBreaker::UnicodeLineBreaker,
        h_align,
        v_align,
      }
    };

    Section {
      screen_position: (0.0, 0.0),
      bounds: (self.max_width.unwrap_or(f32::INFINITY) * 50., self.max_height.unwrap_or(f32::INFINITY) * 50.),
      text: vec![
        self.get_glyph_text(),
      ],
      layout,
      ..Section::default()
    }
  }

  pub fn render(&self, scene: &mut Scene) {
    scene.draw_text(self);
  }
}