use crate::math::utility::saturate;
use num_traits::Num;

fn color_u32_to_color_u8(c: u32) -> (u8, u8, u8, u8) {
  (
    (c >> 24 & 0xFF) as u8,
    (c >> 16 & 0xFF) as u8,
    (c >> 8 & 0xFF) as u8,
    (c & 0xFF) as u8,
  )
}

fn color_u32_to_color_f32(c: u32) -> (f32, f32, f32, f32) {
  (
    (c >> 24 & 0xFF) as f32 / 255_f32,
    (c >> 16 & 0xFF) as f32 / 255_f32,
    (c >> 8 & 0xFF) as f32 / 255_f32,
    (c & 0xff) as f32 / 255_f32,
  )
}

pub trait NumColorComponent<ComponentType = Self> {
  fn alpha_max() -> ComponentType;
  fn from_u32(val: u32) -> (ComponentType, ComponentType, ComponentType, ComponentType);
}

macro_rules! define_color_component {
  ($cctype:ty, $alpha_max:expr, $conv_expr:expr) => {
    impl NumColorComponent for $cctype {

      fn alpha_max() -> $cctype {
        $alpha_max
      }

      fn from_u32(val: u32) -> ($cctype, $cctype, $cctype, $cctype) {
        $conv_expr(val)
      }
    }
  };
}

define_color_component!(u8, 255, color_u32_to_color_u8);
define_color_component!(f32, 1_f32, color_u32_to_color_f32);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TColorRGBA<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent,
{
  pub r: T,
  pub g: T,
  pub b: T,
  pub a: T,
}

impl<T> TColorRGBA<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent,
{
  pub fn new(r: T, g: T, b: T) -> Self {
    Self::new_with_alpha(r, g, b, T::alpha_max())
  }

  pub fn new_with_alpha(r: T, g: T, b: T, a: T) -> Self {
    TColorRGBA { r, g, b, a }
  }

  pub fn from_html(s: &str) -> Result<Self, &'static str> {
    let s = s.trim();
    if s.is_empty() {
      return Err("empty input string");
    }

    let s = if s.starts_with('#') { &s[1..] } else { s };

    let len_content = s.len();
    if !(len_content == 6 || len_content == 8) {
      return Err("wrong component count (either 6 or 8 hex color values expected)");
    }

    u32::from_str_radix(s, 16)
      .map(|color_u32| {
        let color_u32 = if len_content == 6 {
          (color_u32 << 8) | 0xFF
        } else {
          color_u32
        };

        let (r, g, b, a) = T::from_u32(color_u32);
        Self::new_with_alpha(r, g, b, a)
      })
      .map_err(|_| "Invalid color value")
  }

  pub fn as_slice(&self) -> &[T] {
    unsafe { std::slice::from_raw_parts(self as *const TColorRGBA<T> as *const T, 4) }
  }

  pub fn as_slice_mut(&mut self) -> &mut [T] {
    unsafe { std::slice::from_raw_parts_mut(self as *mut Self as *mut T, 4) }
  }
}

pub type RGBAColor = TColorRGBA<u8>;
pub type RGBAColorF32 = TColorRGBA<f32>;

impl<T> std::convert::From<[T; 4]> for TColorRGBA<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent,
{
  fn from(c: [T; 4]) -> Self {
    Self::new_with_alpha(c[0], c[1], c[2], c[3])
  }
}

impl<T> std::convert::From<[T; 3]> for TColorRGBA<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent,
{
  fn from(c: [T; 3]) -> Self {
    Self::new_with_alpha(c[0], c[1], c[2], T::alpha_max())
  }
}

impl std::convert::From<RGBAColor> for RGBAColorF32 {
  fn from(rgba: RGBAColor) -> Self {
    RGBAColorF32::new_with_alpha(
      rgba.r as f32 / 255_f32,
      rgba.g as f32 / 255_f32,
      rgba.b as f32 / 255_f32,
      rgba.a as f32 / 255_f32,
    )
  }
}

impl std::convert::From<RGBAColorF32> for RGBAColor {
  fn from(rgbaf32: RGBAColorF32) -> Self {
    RGBAColor::new_with_alpha(
      (saturate(rgbaf32.r) * 255_f32) as u8,
      (saturate(rgbaf32.g) * 255_f32) as u8,
      (saturate(rgbaf32.b) * 255_f32) as u8,
      (saturate(rgbaf32.a) * 255_f32) as u8,
    )
  }
}

impl std::convert::From<RGBAColor> for u32 {
  fn from(c: RGBAColor) -> u32 {
    (c.r as u32) << 24 | (c.g as u32) << 16 | (c.b as u32) << 8 | (c.a as u32)
  }
}

impl std::convert::From<RGBAColorF32> for u32 {
  fn from(c: RGBAColorF32) -> u32 {
    let r = (saturate(c.r) * 255_f32) as u32;
    let g = (saturate(c.g) * 255_f32) as u32;
    let b = (saturate(c.b) * 255_f32) as u32;
    let a = (saturate(c.a) * 255_f32) as u32;

    (r << 24) | (g << 16) | (b << 8) | a
  }
}

impl<T> std::ops::AddAssign for TColorRGBA<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent + std::ops::AddAssign,
{
  fn add_assign(&mut self, rhs: Self) {
    self
      .as_slice_mut()
      .iter_mut()
      .zip(rhs.as_slice().into_iter())
      .for_each(|(s, r)| *s += *r);
  }
}

impl<T> std::ops::SubAssign for TColorRGBA<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent + std::ops::SubAssign,
{
  fn sub_assign(&mut self, rhs: Self) {
    self
      .as_slice_mut()
      .iter_mut()
      .zip(rhs.as_slice().into_iter())
      .for_each(|(s, r)| *s -= *r);
  }
}

impl<T> std::ops::MulAssign for TColorRGBA<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent + std::ops::MulAssign,
{
  fn mul_assign(&mut self, rhs: Self) {
    self
      .as_slice_mut()
      .iter_mut()
      .zip(rhs.as_slice().into_iter())
      .for_each(|(s, r)| *s *= *r);
  }
}

impl<T> std::ops::MulAssign<T> for TColorRGBA<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent + std::ops::MulAssign,
{
  fn mul_assign(&mut self, k: T) {
    self.as_slice_mut().iter_mut().for_each(|s| *s *= k);
  }
}

impl<T> std::ops::DivAssign for TColorRGBA<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent + std::ops::DivAssign,
{
  fn div_assign(&mut self, rhs: Self) {
    self
      .as_slice_mut()
      .iter_mut()
      .zip(rhs.as_slice().into_iter())
      .for_each(|(s, r)| *s /= *r);
  }
}

impl<T> std::ops::DivAssign<T> for TColorRGBA<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent + std::ops::DivAssign,
{
  fn div_assign(&mut self, k: T) {
    self.as_slice_mut().iter_mut().for_each(|s| *s /= k);
  }
}

impl<T> std::ops::Add for TColorRGBA<T>
where
  T: Copy
    + Clone
    + std::fmt::Debug
    + Num
    + NumColorComponent
    + std::ops::Add<Output = T>
    + std::ops::AddAssign,
{
  type Output = Self;
  fn add(self, rhs: TColorRGBA<T>) -> Self::Output {
    let mut result = self;
    result += rhs;
    result
  }
}

impl<T> std::ops::Sub for TColorRGBA<T>
where
  T: Copy
    + Clone
    + std::fmt::Debug
    + Num
    + NumColorComponent
    + std::ops::Sub<Output = T>
    + std::ops::SubAssign,
{
  type Output = Self;
  fn sub(self, rhs: TColorRGBA<T>) -> Self::Output {
    let mut result = self;
    result -= rhs;
    result
  }
}

impl<T> std::ops::Mul for TColorRGBA<T>
where
  T: Copy
    + Clone
    + std::fmt::Debug
    + Num
    + NumColorComponent
    + std::ops::Mul<Output = T>
    + std::ops::MulAssign,
{
  type Output = Self;
  fn mul(self, rhs: TColorRGBA<T>) -> Self::Output {
    let mut result = self;
    result *= rhs;
    result
  }
}

impl<T> std::ops::Mul<T> for TColorRGBA<T>
where
  T: Copy
    + Clone
    + std::fmt::Debug
    + Num
    + NumColorComponent
    + std::ops::Mul<Output = T>
    + std::ops::MulAssign,
{
  type Output = Self;
  fn mul(self, rhs: T) -> Self::Output {
    let mut result = self;
    result *= rhs;
    result
  }
}

impl<T> std::ops::Div for TColorRGBA<T>
where
  T: Copy
    + Clone
    + std::fmt::Debug
    + Num
    + NumColorComponent
    + std::ops::Div<Output = T>
    + std::ops::DivAssign,
{
  type Output = Self;
  fn div(self, rhs: TColorRGBA<T>) -> Self::Output {
    let mut result = self;
    result /= rhs;
    result
  }
}

impl<T> std::ops::Div<T> for TColorRGBA<T>
where
  T: Copy
    + Clone
    + std::fmt::Debug
    + Num
    + NumColorComponent
    + std::ops::Div<Output = T>
    + std::ops::DivAssign,
{
  type Output = Self;
  fn div(self, rhs: T) -> Self::Output {
    let mut result = self;
    result /= rhs;
    result
  }
}

macro_rules! define_color_type {
    ( $classname:ident, $fieldstype:ty, $numfields:expr, $( ($membername:ident => $initname:ident) ),+ ) => {
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub struct $classname {
            $(
                pub $membername : $fieldstype
            ),+
        }

        impl $classname {
            pub fn new( $($initname : $fieldstype),+ ) -> Self {
                Self {
                    $(
                        $membername : $initname
                    ),+
                }
            }

            pub fn as_slice(&self) -> &[$fieldstype] {
                unsafe {
                    std::slice::from_raw_parts(self as *const _ as *const $fieldstype, $numfields)
                }
            }

            pub fn as_slice_mut(&mut self) -> &mut [$fieldstype] {
                unsafe {
                    std::slice::from_raw_parts_mut(self as *mut _ as *mut $fieldstype, $numfields)
                }
            }
        }
    };
}

define_color_type!(HsvColor, f32, 3usize, (h => hue), (s => saturation), (v => value));
define_color_type!(HslColor, f32, 3usize, (h => hue), (s => lightness), (l => saturation));
define_color_type!(XyzColor, f32, 3usize, (x => xval), (y => yval), (z => zval));

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_conversion_from_html() {

    assert_eq!(
      RGBAColor::from_html("7fdc34"),
      Ok(RGBAColor::new_with_alpha(127, 220, 52, 255))
    );

    assert_eq!(
      RGBAColor::from_html("#7fdc34"),
      Ok(RGBAColor::new_with_alpha(127, 220, 52, 255))
    );

    assert_eq!(
      RGBAColor::from_html("0f499fff"),
      Ok(RGBAColor::new_with_alpha(15, 73, 159, 255))
    );

    assert_eq!(
      RGBAColor::from_html("#0f499fff"),
      Ok(RGBAColor::new_with_alpha(15, 73, 159, 255))
    );

    assert_eq!(RGBAColor::from_html("invalid str").is_ok(), false);
  }

  #[test]
  fn test_conversion_from_slice() {
    let clr = [255u8, 0u8, 128u8, 255u8];
    assert_eq!(
      RGBAColor::from(clr),
      RGBAColor::new_with_alpha(255, 0, 128, 255)
    );

    let c: u32 = RGBAColor::new(0, 51, 153).into();
    assert_eq!(c, 0x003399ff);
  }
}
