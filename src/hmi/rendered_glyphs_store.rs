#![allow(dead_code)]

use crate::hmi::{base::GenericHandle, text_engine::Span};

#[derive(Copy, Debug, Clone)]
pub struct RenderedGlyph {
  pub img:       GenericHandle,
  pub uv_top:    f32,
  pub uv_left:   f32,
  pub uv_right:  f32,
  pub uv_bottom: f32,
}

impl std::default::Default for RenderedGlyph {
  fn default() -> RenderedGlyph {
    Self {
      img:       GenericHandle::Id(0),
      uv_top:    0f32,
      uv_left:   0f32,
      uv_right:  0f32,
      uv_bottom: 0f32,
    }
  }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlyphId {
  pub font_id:   u64,
  pub font_size: i32,
  pub glyph_id:  u32,
}

pub trait RenderedGlyphsStore {
  fn store_glyph(&self, glyph_spans: &[Span]) -> Option<RenderedGlyph>;
}
