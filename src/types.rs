use crate::image::GenericHandle;
use num_traits::{Float, Num};
use std::cmp::PartialOrd;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};


///
/// \brief  Two component vector.
#[derive(Copy, Clone, Debug)]
pub struct TVec2<T> {
  pub x: T,
  pub y: T,
}

impl<T> TVec2<T>
where
  T: Copy + Clone + std::fmt::Debug + Num,
{
  pub fn new(x: T, y: T) -> Self {
    TVec2 { x, y }
  }

  pub fn same(t: T) -> Self {
    Self::new(t, t)
  }

  pub fn as_slice(&self) -> &[T] {
    unsafe { std::slice::from_raw_parts(self as *const Self as *const T, 2) }
  }

  pub fn as_slice_mut(&mut self) -> &mut [T] {
    unsafe { std::slice::from_raw_parts_mut(self as *mut Self as *mut T, 2) }
  }

  pub fn square_len(&self) -> T {
    self.x * self.x + self.y * self.y
  }

  pub fn len(&self) -> T
  where
    T: Float,
  {
    self.square_len().sqrt()
  }
}

impl<T> Into<(T, T)> for TVec2<T>
where
  T: Copy + Clone + std::fmt::Debug,
{
  fn into(self) -> (T, T) {
    (self.x, self.y)
  }
}

///
/// \brief  Negation operator.
impl<T> Neg for TVec2<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + Neg<Output = T>,
{
  type Output = Self;

  fn neg(self) -> Self::Output {
    Self::new(-self.x, -self.y)
  }
}

///
/// \brief  Self-assign addition operator.
impl<T> AddAssign for TVec2<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + AddAssign,
{
  fn add_assign(&mut self, rhs: Self) {
    self.x += rhs.x;
    self.y += rhs.y;
  }
}

///
/// \brief  Addition operator.
impl<T> Add for TVec2<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + Add<Output = T>,
{
  type Output = Self;

  fn add(self, rhs: Self) -> Self {
    Self::new(self.x + rhs.x, self.y + rhs.y)
  }
}

///
/// \brief  Substraction operation.
impl<T> Sub for TVec2<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + Sub<Output = T>,
{
  type Output = Self;
  fn sub(self, rhs: Self) -> Self::Output {
    Self::new(self.x - rhs.x, self.y - rhs.y)
  }
}

///
/// \brief  Self-assign substraction.
impl<T> SubAssign for TVec2<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + SubAssign,
{
  fn sub_assign(&mut self, rhs: Self) {
    self.x -= rhs.x;
    self.y -= rhs.y;
  }
}

///
/// \brief Multiplication with scalar.
impl<T> Mul<T> for TVec2<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + Mul<Output = T>,
{
  type Output = Self;

  fn mul(self, scalar: T) -> Self::Output {
    Self::new(self.x * scalar, self.y * scalar)
  }
}

///
/// \brief Macro to generate scalar with TVec2 multiplication
macro_rules! scalar_multiply_tvec2 {
  ($stype:ty) => {
    impl Mul<TVec2<$stype>> for $stype {
      type Output = TVec2<$stype>;

      fn mul(self, rhs: TVec2<$stype>) -> Self::Output {
        rhs * self
      }
    }
  };
}

scalar_multiply_tvec2!(i8);
scalar_multiply_tvec2!(u8);
scalar_multiply_tvec2!(i16);
scalar_multiply_tvec2!(u16);
scalar_multiply_tvec2!(i32);
scalar_multiply_tvec2!(u32);
scalar_multiply_tvec2!(i64);
scalar_multiply_tvec2!(u64);
scalar_multiply_tvec2!(f32);
scalar_multiply_tvec2!(f64);

///
/// \brief Self-assign scalar multiplication.
impl<T> MulAssign<T> for TVec2<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + MulAssign,
{
  fn mul_assign(&mut self, scalar: T) {
    self.x *= scalar;
    self.y *= scalar;
  }
}

///
/// \brief Component-wise multiplication
impl<T> Mul for TVec2<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + Mul<Output = T>,
{
  type Output = TVec2<T>;

  fn mul(self, rhs: Self) -> Self::Output {
    Self::new(self.x * rhs.x, self.y * rhs.y)
  }
}

///
/// \brief Component-wise self-assign multiplication
impl<T> MulAssign for TVec2<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + MulAssign,
{
  fn mul_assign(&mut self, rhs: TVec2<T>) {
    self.x *= rhs.x;
    self.y *= rhs.y;
  }
}

///
/// \brief Division by scalar.
impl<T> Div<T> for TVec2<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + Div<Output = T>,
{
  type Output = Self;

  fn div(self, scalar: T) -> Self::Output {
    Self::new(self.x / scalar, self.y / scalar)
  }
}

///
/// \brief Component-wise division by another TVec2
impl<T> Div for TVec2<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + Div<Output = T>,
{
  type Output = Self;

  fn div(self, rhs: Self) -> Self::Output {
    Self::new(self.x / rhs.x, self.y / rhs.y)
  }
}

///
/// \brief Self-assign division by scalar.
impl<T> DivAssign<T> for TVec2<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + DivAssign,
{

  fn div_assign(&mut self, scalar: T) {
    self.x /= scalar;
    self.y /= scalar;
  }
}

///
/// \brief Self-assign division by another TVec2.
impl<T> DivAssign for TVec2<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + DivAssign,
{
  fn div_assign(&mut self, rhs: Self) {
    self.x /= rhs.x;
    self.y /= rhs.y;
  }
}

///
/// @{ Operations on TVec2

///
/// \brief  Normalizes the input vector.
pub fn normalize<T>(a: TVec2<T>) -> TVec2<T>
where
  T: Copy + Clone + std::fmt::Debug + Float,
{
  let square_len = a.square_len();
  if square_len.is_zero() {
    a
  } else {
    a * square_len.sqrt().recip()
  }
}

pub fn is_unit_length<T>(a: TVec2<T>) -> bool
where
  T: Copy + Clone + std::fmt::Debug + Num,
{
  a.square_len() == T::one()
}

///
/// \brief  The dot product of two vectors.
pub fn dot<T>(a: TVec2<T>, b: TVec2<T>) -> T
where
  T: Copy + Clone + std::fmt::Debug + Num,
{
  a.x * b.x + a.y * b.y
}

///
/// \brief  Returns a vector that is perpendicular to the input vector by applying a CCW PI/2 rotation.
pub fn perp_vec<T>(a: TVec2<T>) -> TVec2<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + Neg<Output = T>,
{
  TVec2::new(-a.y, a.x)
}

///
/// \brief  Returns the perp product of two vectors. Given the vectors a and b, the formula for the
/// perp product is dot(a, perp(b))
pub fn perp<T>(a: TVec2<T>, b: TVec2<T>) -> T
where
  T: Copy + Clone + std::fmt::Debug + Num + Neg<Output = T>,
{
  -a.x * b.y + a.y * b.x
}

pub fn are_orthogonal<T>(a: TVec2<T>, b: TVec2<T>) -> bool
where
  T: Copy + Clone + std::fmt::Debug + Num,
{
  dot(a, b).is_zero()
}

///
/// @}

///
/// @{
///

pub type Vec2I8 = TVec2<i8>;
pub type Vec2U8 = TVec2<u8>;
pub type Vec2I16 = TVec2<i16>;
pub type Vec2U16 = TVec2<u16>;
pub type Vec2I32 = TVec2<i32>;
pub type Vec2U32 = TVec2<u32>;
pub type Vec2F32 = TVec2<f32>;

///
/// @}
///

pub fn saturate(x: f32) -> f32 {
  (x.min(1_f32)).max(0_f32)
}

pub fn clamp(minval: f32, x: f32, maxval: f32) -> f32 {
  minval.max(x).min(maxval)
}

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
pub struct DrawNullTexture {
  ///<! texture handle to a texture containing a white pixel
  pub texture: GenericHandle,
  ///<! Coordinates of the white pixel in the above texture
  pub uv: Vec2F32,
}

#[derive(Debug, Clone)]
pub struct ConvertConfig {
  pub global_alpha: f32,
  pub line_aa: AntialiasingType,
  pub shape_aa: AntialiasingType,
  pub circle_segment_count: u32,
  pub arc_segment_count: u32,
  pub curve_segment_count: u32,
  pub null: DrawNullTexture,
  pub vertex_layout: Vec<DrawVertexLayoutElement>,
  pub vertex_size: usize,
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

impl std::default::Default for VertexPTC {
  fn default() -> Self {
    VertexPTC {
      pos: Vec2F32::new(0_f32, 0_f32),
      texcoords: Vec2F32::new(0_f32, 0_f32),
      color: RGBAColorF32::new(0_f32, 0_f32, 0_f32, 1_f32),
    }
  }
}

#[derive(Copy, Debug, Clone, Eq, PartialEq)]
pub enum AntialiasingType {
  Off,
  On,
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum DrawVertexLayoutAttribute {
  Position,
  Color,
  Texcoord,
  // Count,
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum DrawVertexLayoutFormat {
  Schar,
  Sshort,
  Sint,
  Uchar,
  Ushort,
  Uint,
  Float,
  Double,
  FormatColorBegin,
  R8G8B8,
  R16G15B16,
  R32G32B32,
  R8G8B8A8,
  B8G8R8A8,
  R16G15B16A16,
  R32G32B32A32,
  R32G32B32A32_Float,
  R32G32B32A32_Double,
  RGB32,
  RGBA32,
  FormatColorEnd,
  FormatCount,
}

#[derive(Copy, Debug, Clone)]
pub struct DrawVertexLayoutElement {
  pub attribute: DrawVertexLayoutAttribute,
  pub format: DrawVertexLayoutFormat,
  pub offset: usize,
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