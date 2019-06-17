
use std::cmp::PartialEq;
use std::cmp::PartialOrd;
use std::ops::Add;
#[derive(Copy, Clone, Debug)]
pub struct TVec2<T>
where
  T: Copy + Clone + std::fmt::Debug,
{
  pub x: T,
  pub y: T,
}

impl<T> TVec2<T>
where
  T: Copy + Clone + std::fmt::Debug,
{
  pub fn new(x: T, y: T) -> Self {
    TVec2 { x, y }
  }
}

pub type Vec2F32 = TVec2<f32>;
pub type Vec2I16 = TVec2<i16>;

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

  pub fn as_slice(&self) -> &[T] {
    unsafe { std::slice::from_raw_parts(self as *const TColorRGBA<T> as *const T, 4) }
  }
}

pub type RGBAColor = TColorRGBA<u8>;
pub type RGBAColorF32 = TColorRGBA<f32>;

#[derive(Copy, Clone, Debug)]
pub struct TRectangle<T>
where
  T: Copy + Clone + std::fmt::Debug,
{
  pub x: T,
  pub y: T,
  pub w: T,
  pub h: T,
}

impl<T> TRectangle<T>
where
  T: Copy + Clone + std::fmt::Debug,
{
  pub fn new(x: T, y: T, width: T, height: T) -> Self {
    TRectangle {
      x,
      y,
      w: width,
      h: height,
    }
  }

  pub fn intersect(&self, rhs: &TRectangle<T>) -> bool
  where
    T: Add<Output = T> + PartialEq + PartialOrd,
  {
    !((self.x + self.w) < rhs.x
      || (rhs.x + rhs.w) < self.x
      || (self.y + self.h) < rhs.y
      || (rhs.y + rhs.h) < self.y)
  }

  pub fn contains_point(&self, x: T, y: T) -> bool
  where
    T: Add<Output = T> + PartialEq + PartialOrd,
  {
    x >= self.x && x <= (self.x + self.w) && y >= self.y && y <= (self.y + self.h)
  }
}

pub type RectangleI16 = TRectangle<i16>;
pub type RectangleF32 = TRectangle<f32>;

#[derive(Copy, Debug, Clone)]
pub enum GenericHandle {
  Ptr(usize),
  Id(i32),
}

#[derive(Copy, Debug, Clone)]
pub struct Image {
  handle: GenericHandle,
  w: u16,
  h: u16,
  region: [u16; 4],
}

#[derive(Copy, Debug, Clone)]
pub struct DrawNullTexture {
  ///<! texture handle to a texture containing a white pixel
  pub texture: GenericHandle,
  ///<! Coordinates of the white pixel in the above texture
  pub uv: Vec2F32,
}

#[derive(Debug, Clone)]
pub struct ConvertConfig {
  global_alpha : f32,
  line_aa : AntialiasingType,
  shape_aa : AntialiasingType,
  circle_segment_count : u32,
  arc_segment_count : u32,
  curve_segment_count : u32,
  null : DrawNullTexture,
  vertex_layout : Vec<i32>,

}

#[derive(Copy, Debug, Clone)]
pub struct UserFont {}

#[derive(Copy, Debug, Clone)]
pub struct PlaceholderType {}

#[derive(Copy, Debug, Clone)]
pub struct VertexPTC {
  pub pos: Vec2F32,
  pub texcoords: Vec2F32,
  pub color: RGBAColorF32,
}

#[derive(Copy, Debug, Clone)]
pub enum AntialiasingType {
  Off,
  On,
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_rgba() {
    let clr = RGBAColor::new(255, 64, 128, 255);
    let s = clr.as_slice();
    assert_eq!(clr.r, s[0]);
    assert_eq!(clr.g, s[1]);
    assert_eq!(clr.b, s[2]);
    assert_eq!(clr.a, s[3]);
  }
}