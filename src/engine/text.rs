mod cg_text;
mod text_renderer;
mod font;
mod text_pipeline;
mod text_vertex;

pub use text_renderer:: {
  TextRenderer,
  DrawText
};
pub use cg_text::{
  CGText,
  TextAlignment,
  TextWrapStyle,
  VerticalTextAlignment,
};
pub use font::FontFace;