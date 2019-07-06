use std::ops::Add;

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
    x >= self.x
      && x <= (self.x + self.w)
      && y >= self.y
      && y <= (self.y + self.h)
  }
}

pub type RectangleI16 = TRectangle<i16>;
pub type RectangleF32 = TRectangle<f32>;
