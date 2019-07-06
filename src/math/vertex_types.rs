use crate::math::{colors::RGBAColorF32, vec2::Vec2F32};

#[derive(Copy, Debug, Clone)]
pub struct VertexPTC {
  pub pos:       Vec2F32,
  pub texcoords: Vec2F32,
  pub color:     RGBAColorF32,
}

impl std::default::Default for VertexPTC {
  fn default() -> Self {
    VertexPTC {
      pos:       Vec2F32::new(0_f32, 0_f32),
      texcoords: Vec2F32::new(0_f32, 0_f32),
      color:     RGBAColorF32::new(0_f32, 0_f32, 0_f32),
    }
  }
}
