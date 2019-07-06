use num_traits::{Float, Num};
use std::ops::{
  Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign,
};

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

/// @{ Operations on TVec2

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

/// \brief  The dot product of two vectors.
pub fn dot<T>(a: TVec2<T>, b: TVec2<T>) -> T
where
  T: Copy + Clone + std::fmt::Debug + Num,
{
  a.x * b.x + a.y * b.y
}

/// \brief  Returns a vector that is perpendicular to the input vector by
/// applying a CCW PI/2 rotation.
pub fn perp_vec<T>(a: TVec2<T>) -> TVec2<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + Neg<Output = T>,
{
  TVec2::new(-a.y, a.x)
}

/// \brief  Returns the perp product of two vectors. Given the vectors a and b,
/// the formula for the perp product is dot(a, perp(b))
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

/// @}

pub type Vec2I8 = TVec2<i8>;
pub type Vec2U8 = TVec2<u8>;
pub type Vec2I16 = TVec2<i16>;
pub type Vec2U16 = TVec2<u16>;
pub type Vec2I32 = TVec2<i32>;
pub type Vec2U32 = TVec2<u32>;
pub type Vec2F32 = TVec2<f32>;
