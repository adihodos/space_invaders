use crate::{hmi::image::Image, math::vec2::Vec2F32};

#[derive(Copy, Clone, Debug)]
pub struct Cursor {
  pub img:    Image,
  pub size:   Vec2F32,
  pub offset: Vec2F32,
}
