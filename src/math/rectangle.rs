#![allow(dead_code)]

use num_traits::Num;
use std::ops::{Add, Sub};

use crate::math::minmax::MinMax;

#[derive(Copy, Clone, Debug)]
pub struct TRectangle<T>
where
  T: Copy + Clone + std::fmt::Debug + Num,
{
  pub x: T,
  pub y: T,
  pub w: T,
  pub h: T,
}

impl<T> TRectangle<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + MinMax<Output = T>,
{
  pub fn new(x: T, y: T, width: T, height: T) -> Self {
    TRectangle {
      x,
      y,
      w: width,
      h: height,
    }
  }

  pub fn from_points(x0: T, y0: T, x1: T, y1: T) -> Self
  where
    T: Sub<Output = T>,
  {
    Self::new(x0, y0, x1 - x0, y1 - y0)
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
    x >= self.x
      && x <= (self.x + self.w)
      && y >= self.y
      && y <= (self.y + self.h)
  }

  pub fn union(a: &TRectangle<T>, b: &TRectangle<T>) -> TRectangle<T>
  where
    T: PartialOrd,
  {
    Self::new(
      <T as MinMax>::min(a.x, b.x),
      <T as MinMax>::min(a.y, b.y),
      <T as MinMax>::max(a.w, b.w),
      <T as MinMax>::max(a.h, b.h),
    )
  }
}

pub type RectangleI16 = TRectangle<i16>;
pub type RectangleI32 = TRectangle<i32>;
pub type RectangleF32 = TRectangle<f32>;
