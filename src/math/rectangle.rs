#![allow(dead_code)]

use num_traits::Num;
use std::ops::{Add, Sub};

use crate::math::{minmax::MinMax, vec2::Vec2};

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
    T: PartialOrd + Add<Output = T> + Sub<Output = T>,
  {
    let ax0 = a.x;
    let ay0 = a.y;
    let ax1 = ax0 + a.w;
    let ay1 = ay0 + a.h;

    let bx0 = b.x;
    let by0 = b.y;
    let bx1 = bx0 + b.w;
    let by1 = by0 + b.h;

    let ux0 = <T as MinMax>::min(ax0, bx0);
    let uy0 = <T as MinMax>::min(ay0, by0);
    let ux1 = <T as MinMax>::max(ax1, bx1);
    let uy1 = <T as MinMax>::max(ay1, by1);

    Self::new(ux0, uy0, ux1 - ux0, uy1 - uy0)
  }

  pub fn shrink(r: &TRectangle<T>, amount: T) -> TRectangle<T>
  where
    T: Add<Output = T> + Sub<Output = T> + MinMax,
  {
    let w = T::max(r.w, amount + amount);
    let h = T::max(r.h, amount + amount);

    TRectangle::new(
      r.x + amount,
      r.y + amount,
      w - amount - amount,
      h - amount - amount,
    )
  }

  pub fn pad(r: &TRectangle<T>, pad: Vec2<T>) -> TRectangle<T> {
    let w = T::max(r.w, pad.x + pad.x);
    let h = T::max(r.h, pad.y + pad.y);
    TRectangle::new(
      r.x + pad.x,
      r.y + pad.y,
      w - pad.x - pad.x,
      h - pad.y - pad.y,
    )
  }
}

pub type RectangleI16 = TRectangle<i16>;
pub type RectangleI32 = TRectangle<i32>;
pub type RectangleF32 = TRectangle<f32>;
