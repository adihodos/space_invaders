use crate::math::utility::saturate;

#[derive(Copy, Clone, Debug)]
pub struct TColorRGBA<T>
where
  T: Copy + Clone + std::fmt::Debug,
{
  pub r: T,
  pub g: T,
  pub b: T,
  pub a: T,
}

impl<T> TColorRGBA<T>
where
  T: Copy + Clone + std::fmt::Debug,
{
  pub fn new(r: T, g: T, b: T, a: T) -> Self {
    TColorRGBA { r, g, b, a }
  }

  pub fn from_slice(c: &[T]) -> Self {
    assert!(c.len() == 4);
    TColorRGBA {
      r: c[0],
      g: c[1],
      b: c[2],
      a: c[3],
    }
  }

  pub fn as_slice(&self) -> &[T] {
    unsafe { std::slice::from_raw_parts(self as *const TColorRGBA<T> as *const T, 4) }
  }
}

pub type RGBAColor = TColorRGBA<u8>;
pub type RGBAColorF32 = TColorRGBA<f32>;

impl std::convert::From<RGBAColor> for RGBAColorF32 {
  fn from(rgba: RGBAColor) -> Self {
    RGBAColorF32::new(
      rgba.r as f32 / 255_f32,
      rgba.g as f32 / 255_f32,
      rgba.b as f32 / 255_f32,
      rgba.a as f32 / 255_f32,
    )
  }
}

impl std::convert::From<RGBAColorF32> for RGBAColor {
  fn from(rgbaf32: RGBAColorF32) -> Self {
    RGBAColor::new(
      (saturate(rgbaf32.r) * 255_f32) as u8,
      (saturate(rgbaf32.g) * 255_f32) as u8,
      (saturate(rgbaf32.b) * 255_f32) as u8,
      (saturate(rgbaf32.a) * 255_f32) as u8,
    )
  }
}

pub fn rgba_color_f32_to_rgba_color(c: RGBAColorF32) -> RGBAColor {
  RGBAColor::new(
    (saturate(c.r) * 255_f32) as u8,
    (saturate(c.g) * 255_f32) as u8,
    (saturate(c.b) * 255_f32) as u8,
    (saturate(c.a) * 255_f32) as u8,
  )
}

pub fn rgba_color_to_u32(c: RGBAColor) -> u32 {
  let mut out = c.r as u32;
  out |= (c.g as u32) << 8;
  out |= (c.b as u32) << 16;
  out |= (c.a as u32) << 24;
  return out;
}

