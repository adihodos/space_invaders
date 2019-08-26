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
  fn from_u32(
    val: u32,
  ) -> (ComponentType, ComponentType, ComponentType, ComponentType);
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

    let s = if s.starts_with('#') { &s[1 ..] } else { s };

    let len_content = s.len();
    if !(len_content == 6 || len_content == 8) {
      return Err(
        "wrong component count (either 6 or 8 hex color values expected)",
      );
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
    unsafe {
      std::slice::from_raw_parts(self as *const TColorRGBA<T> as *const T, 4)
    }
  }

  pub fn as_slice_mut(&mut self) -> &mut [T] {
    unsafe { std::slice::from_raw_parts_mut(self as *mut Self as *mut T, 4) }
  }
}

pub type RGBAColor = TColorRGBA<u8>;
pub type RGBAColorF32 = TColorRGBA<f32>;

impl<T> std::default::Default for TColorRGBA<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent,
{
  fn default() -> Self {
    Self::new_with_alpha(T::zero(), T::zero(), T::zero(), T::zero())
  }
}

impl<T> std::convert::From<(T, T, T, T)> for TColorRGBA<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent,
{
  fn from(c: (T, T, T, T)) -> Self {
    Self::new_with_alpha(c.0, c.1, c.2, c.3)
  }
}

impl<T> std::convert::From<(T, T, T)> for TColorRGBA<T>
where
  T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent,
{
  fn from(c: (T, T, T)) -> Self {
    Self::new_with_alpha(c.0, c.1, c.2, T::alpha_max())
  }
}

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
  T: Copy
    + Clone
    + std::fmt::Debug
    + Num
    + NumColorComponent
    + std::ops::AddAssign,
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
  T: Copy
    + Clone
    + std::fmt::Debug
    + Num
    + NumColorComponent
    + std::ops::SubAssign,
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
  T: Copy
    + Clone
    + std::fmt::Debug
    + Num
    + NumColorComponent
    + std::ops::MulAssign,
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
  T: Copy
    + Clone
    + std::fmt::Debug
    + Num
    + NumColorComponent
    + std::ops::MulAssign,
{
  fn mul_assign(&mut self, k: T) {
    self.as_slice_mut().iter_mut().for_each(|s| *s *= k);
  }
}

impl<T> std::ops::DivAssign for TColorRGBA<T>
where
  T: Copy
    + Clone
    + std::fmt::Debug
    + Num
    + NumColorComponent
    + std::ops::DivAssign,
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
  T: Copy
    + Clone
    + std::fmt::Debug
    + Num
    + NumColorComponent
    + std::ops::DivAssign,
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

pub const TURBO_SRGB_FLOATS: [[f32; 3]; 256] = [
  [0.18995_f32, 0.07176_f32, 0.23217_f32],
  [0.19483_f32, 0.08339_f32, 0.26149_f32],
  [0.19956_f32, 0.09498_f32, 0.29024_f32],
  [0.20415_f32, 0.10652_f32, 0.31844_f32],
  [0.20860_f32, 0.11802_f32, 0.34607_f32],
  [0.21291_f32, 0.12947_f32, 0.37314_f32],
  [0.21708_f32, 0.14087_f32, 0.39964_f32],
  [0.22111_f32, 0.15223_f32, 0.42558_f32],
  [0.22500_f32, 0.16354_f32, 0.45096_f32],
  [0.22875_f32, 0.17481_f32, 0.47578_f32],
  [0.23236_f32, 0.18603_f32, 0.50004_f32],
  [0.23582_f32, 0.19720_f32, 0.52373_f32],
  [0.23915_f32, 0.20833_f32, 0.54686_f32],
  [0.24234_f32, 0.21941_f32, 0.56942_f32],
  [0.24539_f32, 0.23044_f32, 0.59142_f32],
  [0.24830_f32, 0.24143_f32, 0.61286_f32],
  [0.25107_f32, 0.25237_f32, 0.63374_f32],
  [0.25369_f32, 0.26327_f32, 0.65406_f32],
  [0.25618_f32, 0.27412_f32, 0.67381_f32],
  [0.25853_f32, 0.28492_f32, 0.69300_f32],
  [0.26074_f32, 0.29568_f32, 0.71162_f32],
  [0.26280_f32, 0.30639_f32, 0.72968_f32],
  [0.26473_f32, 0.31706_f32, 0.74718_f32],
  [0.26652_f32, 0.32768_f32, 0.76412_f32],
  [0.26816_f32, 0.33825_f32, 0.78050_f32],
  [0.26967_f32, 0.34878_f32, 0.79631_f32],
  [0.27103_f32, 0.35926_f32, 0.81156_f32],
  [0.27226_f32, 0.36970_f32, 0.82624_f32],
  [0.27334_f32, 0.38008_f32, 0.84037_f32],
  [0.27429_f32, 0.39043_f32, 0.85393_f32],
  [0.27509_f32, 0.40072_f32, 0.86692_f32],
  [0.27576_f32, 0.41097_f32, 0.87936_f32],
  [0.27628_f32, 0.42118_f32, 0.89123_f32],
  [0.27667_f32, 0.43134_f32, 0.90254_f32],
  [0.27691_f32, 0.44145_f32, 0.91328_f32],
  [0.27701_f32, 0.45152_f32, 0.92347_f32],
  [0.27698_f32, 0.46153_f32, 0.93309_f32],
  [0.27680_f32, 0.47151_f32, 0.94214_f32],
  [0.27648_f32, 0.48144_f32, 0.95064_f32],
  [0.27603_f32, 0.49132_f32, 0.95857_f32],
  [0.27543_f32, 0.50115_f32, 0.96594_f32],
  [0.27469_f32, 0.51094_f32, 0.97275_f32],
  [0.27381_f32, 0.52069_f32, 0.97899_f32],
  [0.27273_f32, 0.53040_f32, 0.98461_f32],
  [0.27106_f32, 0.54015_f32, 0.98930_f32],
  [0.26878_f32, 0.54995_f32, 0.99303_f32],
  [0.26592_f32, 0.55979_f32, 0.99583_f32],
  [0.26252_f32, 0.56967_f32, 0.99773_f32],
  [0.25862_f32, 0.57958_f32, 0.99876_f32],
  [0.25425_f32, 0.58950_f32, 0.99896_f32],
  [0.24946_f32, 0.59943_f32, 0.99835_f32],
  [0.24427_f32, 0.60937_f32, 0.99697_f32],
  [0.23874_f32, 0.61931_f32, 0.99485_f32],
  [0.23288_f32, 0.62923_f32, 0.99202_f32],
  [0.22676_f32, 0.63913_f32, 0.98851_f32],
  [0.22039_f32, 0.64901_f32, 0.98436_f32],
  [0.21382_f32, 0.65886_f32, 0.97959_f32],
  [0.20708_f32, 0.66866_f32, 0.97423_f32],
  [0.20021_f32, 0.67842_f32, 0.96833_f32],
  [0.19326_f32, 0.68812_f32, 0.96190_f32],
  [0.18625_f32, 0.69775_f32, 0.95498_f32],
  [0.17923_f32, 0.70732_f32, 0.94761_f32],
  [0.17223_f32, 0.71680_f32, 0.93981_f32],
  [0.16529_f32, 0.72620_f32, 0.93161_f32],
  [0.15844_f32, 0.73551_f32, 0.92305_f32],
  [0.15173_f32, 0.74472_f32, 0.91416_f32],
  [0.14519_f32, 0.75381_f32, 0.90496_f32],
  [0.13886_f32, 0.76279_f32, 0.89550_f32],
  [0.13278_f32, 0.77165_f32, 0.88580_f32],
  [0.12698_f32, 0.78037_f32, 0.87590_f32],
  [0.12151_f32, 0.78896_f32, 0.86581_f32],
  [0.11639_f32, 0.79740_f32, 0.85559_f32],
  [0.11167_f32, 0.80569_f32, 0.84525_f32],
  [0.10738_f32, 0.81381_f32, 0.83484_f32],
  [0.10357_f32, 0.82177_f32, 0.82437_f32],
  [0.10026_f32, 0.82955_f32, 0.81389_f32],
  [0.09750_f32, 0.83714_f32, 0.80342_f32],
  [0.09532_f32, 0.84455_f32, 0.79299_f32],
  [0.09377_f32, 0.85175_f32, 0.78264_f32],
  [0.09287_f32, 0.85875_f32, 0.77240_f32],
  [0.09267_f32, 0.86554_f32, 0.76230_f32],
  [0.09320_f32, 0.87211_f32, 0.75237_f32],
  [0.09451_f32, 0.87844_f32, 0.74265_f32],
  [0.09662_f32, 0.88454_f32, 0.73316_f32],
  [0.09958_f32, 0.89040_f32, 0.72393_f32],
  [0.10342_f32, 0.89600_f32, 0.71500_f32],
  [0.10815_f32, 0.90142_f32, 0.70599_f32],
  [0.11374_f32, 0.90673_f32, 0.69651_f32],
  [0.12014_f32, 0.91193_f32, 0.68660_f32],
  [0.12733_f32, 0.91701_f32, 0.67627_f32],
  [0.13526_f32, 0.92197_f32, 0.66556_f32],
  [0.14391_f32, 0.92680_f32, 0.65448_f32],
  [0.15323_f32, 0.93151_f32, 0.64308_f32],
  [0.16319_f32, 0.93609_f32, 0.63137_f32],
  [0.17377_f32, 0.94053_f32, 0.61938_f32],
  [0.18491_f32, 0.94484_f32, 0.60713_f32],
  [0.19659_f32, 0.94901_f32, 0.59466_f32],
  [0.20877_f32, 0.95304_f32, 0.58199_f32],
  [0.22142_f32, 0.95692_f32, 0.56914_f32],
  [0.23449_f32, 0.96065_f32, 0.55614_f32],
  [0.24797_f32, 0.96423_f32, 0.54303_f32],
  [0.26180_f32, 0.96765_f32, 0.52981_f32],
  [0.27597_f32, 0.97092_f32, 0.51653_f32],
  [0.29042_f32, 0.97403_f32, 0.50321_f32],
  [0.30513_f32, 0.97697_f32, 0.48987_f32],
  [0.32006_f32, 0.97974_f32, 0.47654_f32],
  [0.33517_f32, 0.98234_f32, 0.46325_f32],
  [0.35043_f32, 0.98477_f32, 0.45002_f32],
  [0.36581_f32, 0.98702_f32, 0.43688_f32],
  [0.38127_f32, 0.98909_f32, 0.42386_f32],
  [0.39678_f32, 0.99098_f32, 0.41098_f32],
  [0.41229_f32, 0.99268_f32, 0.39826_f32],
  [0.42778_f32, 0.99419_f32, 0.38575_f32],
  [0.44321_f32, 0.99551_f32, 0.37345_f32],
  [0.45854_f32, 0.99663_f32, 0.36140_f32],
  [0.47375_f32, 0.99755_f32, 0.34963_f32],
  [0.48879_f32, 0.99828_f32, 0.33816_f32],
  [0.50362_f32, 0.99879_f32, 0.32701_f32],
  [0.51822_f32, 0.99910_f32, 0.31622_f32],
  [0.53255_f32, 0.99919_f32, 0.30581_f32],
  [0.54658_f32, 0.99907_f32, 0.29581_f32],
  [0.56026_f32, 0.99873_f32, 0.28623_f32],
  [0.57357_f32, 0.99817_f32, 0.27712_f32],
  [0.58646_f32, 0.99739_f32, 0.26849_f32],
  [0.59891_f32, 0.99638_f32, 0.26038_f32],
  [0.61088_f32, 0.99514_f32, 0.25280_f32],
  [0.62233_f32, 0.99366_f32, 0.24579_f32],
  [0.63323_f32, 0.99195_f32, 0.23937_f32],
  [0.64362_f32, 0.98999_f32, 0.23356_f32],
  [0.65394_f32, 0.98775_f32, 0.22835_f32],
  [0.66428_f32, 0.98524_f32, 0.22370_f32],
  [0.67462_f32, 0.98246_f32, 0.21960_f32],
  [0.68494_f32, 0.97941_f32, 0.21602_f32],
  [0.69525_f32, 0.97610_f32, 0.21294_f32],
  [0.70553_f32, 0.97255_f32, 0.21032_f32],
  [0.71577_f32, 0.96875_f32, 0.20815_f32],
  [0.72596_f32, 0.96470_f32, 0.20640_f32],
  [0.73610_f32, 0.96043_f32, 0.20504_f32],
  [0.74617_f32, 0.95593_f32, 0.20406_f32],
  [0.75617_f32, 0.95121_f32, 0.20343_f32],
  [0.76608_f32, 0.94627_f32, 0.20311_f32],
  [0.77591_f32, 0.94113_f32, 0.20310_f32],
  [0.78563_f32, 0.93579_f32, 0.20336_f32],
  [0.79524_f32, 0.93025_f32, 0.20386_f32],
  [0.80473_f32, 0.92452_f32, 0.20459_f32],
  [0.81410_f32, 0.91861_f32, 0.20552_f32],
  [0.82333_f32, 0.91253_f32, 0.20663_f32],
  [0.83241_f32, 0.90627_f32, 0.20788_f32],
  [0.84133_f32, 0.89986_f32, 0.20926_f32],
  [0.85010_f32, 0.89328_f32, 0.21074_f32],
  [0.85868_f32, 0.88655_f32, 0.21230_f32],
  [0.86709_f32, 0.87968_f32, 0.21391_f32],
  [0.87530_f32, 0.87267_f32, 0.21555_f32],
  [0.88331_f32, 0.86553_f32, 0.21719_f32],
  [0.89112_f32, 0.85826_f32, 0.21880_f32],
  [0.89870_f32, 0.85087_f32, 0.22038_f32],
  [0.90605_f32, 0.84337_f32, 0.22188_f32],
  [0.91317_f32, 0.83576_f32, 0.22328_f32],
  [0.92004_f32, 0.82806_f32, 0.22456_f32],
  [0.92666_f32, 0.82025_f32, 0.22570_f32],
  [0.93301_f32, 0.81236_f32, 0.22667_f32],
  [0.93909_f32, 0.80439_f32, 0.22744_f32],
  [0.94489_f32, 0.79634_f32, 0.22800_f32],
  [0.95039_f32, 0.78823_f32, 0.22831_f32],
  [0.95560_f32, 0.78005_f32, 0.22836_f32],
  [0.96049_f32, 0.77181_f32, 0.22811_f32],
  [0.96507_f32, 0.76352_f32, 0.22754_f32],
  [0.96931_f32, 0.75519_f32, 0.22663_f32],
  [0.97323_f32, 0.74682_f32, 0.22536_f32],
  [0.97679_f32, 0.73842_f32, 0.22369_f32],
  [0.98000_f32, 0.73000_f32, 0.22161_f32],
  [0.98289_f32, 0.72140_f32, 0.21918_f32],
  [0.98549_f32, 0.71250_f32, 0.21650_f32],
  [0.98781_f32, 0.70330_f32, 0.21358_f32],
  [0.98986_f32, 0.69382_f32, 0.21043_f32],
  [0.99163_f32, 0.68408_f32, 0.20706_f32],
  [0.99314_f32, 0.67408_f32, 0.20348_f32],
  [0.99438_f32, 0.66386_f32, 0.19971_f32],
  [0.99535_f32, 0.65341_f32, 0.19577_f32],
  [0.99607_f32, 0.64277_f32, 0.19165_f32],
  [0.99654_f32, 0.63193_f32, 0.18738_f32],
  [0.99675_f32, 0.62093_f32, 0.18297_f32],
  [0.99672_f32, 0.60977_f32, 0.17842_f32],
  [0.99644_f32, 0.59846_f32, 0.17376_f32],
  [0.99593_f32, 0.58703_f32, 0.16899_f32],
  [0.99517_f32, 0.57549_f32, 0.16412_f32],
  [0.99419_f32, 0.56386_f32, 0.15918_f32],
  [0.99297_f32, 0.55214_f32, 0.15417_f32],
  [0.99153_f32, 0.54036_f32, 0.14910_f32],
  [0.98987_f32, 0.52854_f32, 0.14398_f32],
  [0.98799_f32, 0.51667_f32, 0.13883_f32],
  [0.98590_f32, 0.50479_f32, 0.13367_f32],
  [0.98360_f32, 0.49291_f32, 0.12849_f32],
  [0.98108_f32, 0.48104_f32, 0.12332_f32],
  [0.97837_f32, 0.46920_f32, 0.11817_f32],
  [0.97545_f32, 0.45740_f32, 0.11305_f32],
  [0.97234_f32, 0.44565_f32, 0.10797_f32],
  [0.96904_f32, 0.43399_f32, 0.10294_f32],
  [0.96555_f32, 0.42241_f32, 0.09798_f32],
  [0.96187_f32, 0.41093_f32, 0.09310_f32],
  [0.95801_f32, 0.39958_f32, 0.08831_f32],
  [0.95398_f32, 0.38836_f32, 0.08362_f32],
  [0.94977_f32, 0.37729_f32, 0.07905_f32],
  [0.94538_f32, 0.36638_f32, 0.07461_f32],
  [0.94084_f32, 0.35566_f32, 0.07031_f32],
  [0.93612_f32, 0.34513_f32, 0.06616_f32],
  [0.93125_f32, 0.33482_f32, 0.06218_f32],
  [0.92623_f32, 0.32473_f32, 0.05837_f32],
  [0.92105_f32, 0.31489_f32, 0.05475_f32],
  [0.91572_f32, 0.30530_f32, 0.05134_f32],
  [0.91024_f32, 0.29599_f32, 0.04814_f32],
  [0.90463_f32, 0.28696_f32, 0.04516_f32],
  [0.89888_f32, 0.27824_f32, 0.04243_f32],
  [0.89298_f32, 0.26981_f32, 0.03993_f32],
  [0.88691_f32, 0.26152_f32, 0.03753_f32],
  [0.88066_f32, 0.25334_f32, 0.03521_f32],
  [0.87422_f32, 0.24526_f32, 0.03297_f32],
  [0.86760_f32, 0.23730_f32, 0.03082_f32],
  [0.86079_f32, 0.22945_f32, 0.02875_f32],
  [0.85380_f32, 0.22170_f32, 0.02677_f32],
  [0.84662_f32, 0.21407_f32, 0.02487_f32],
  [0.83926_f32, 0.20654_f32, 0.02305_f32],
  [0.83172_f32, 0.19912_f32, 0.02131_f32],
  [0.82399_f32, 0.19182_f32, 0.01966_f32],
  [0.81608_f32, 0.18462_f32, 0.01809_f32],
  [0.80799_f32, 0.17753_f32, 0.01660_f32],
  [0.79971_f32, 0.17055_f32, 0.01520_f32],
  [0.79125_f32, 0.16368_f32, 0.01387_f32],
  [0.78260_f32, 0.15693_f32, 0.01264_f32],
  [0.77377_f32, 0.15028_f32, 0.01148_f32],
  [0.76476_f32, 0.14374_f32, 0.01041_f32],
  [0.75556_f32, 0.13731_f32, 0.00942_f32],
  [0.74617_f32, 0.13098_f32, 0.00851_f32],
  [0.73661_f32, 0.12477_f32, 0.00769_f32],
  [0.72686_f32, 0.11867_f32, 0.00695_f32],
  [0.71692_f32, 0.11268_f32, 0.00629_f32],
  [0.70680_f32, 0.10680_f32, 0.00571_f32],
  [0.69650_f32, 0.10102_f32, 0.00522_f32],
  [0.68602_f32, 0.09536_f32, 0.00481_f32],
  [0.67535_f32, 0.08980_f32, 0.00449_f32],
  [0.66449_f32, 0.08436_f32, 0.00424_f32],
  [0.65345_f32, 0.07902_f32, 0.00408_f32],
  [0.64223_f32, 0.07380_f32, 0.00401_f32],
  [0.63082_f32, 0.06868_f32, 0.00401_f32],
  [0.61923_f32, 0.06367_f32, 0.00410_f32],
  [0.60746_f32, 0.05878_f32, 0.00427_f32],
  [0.59550_f32, 0.05399_f32, 0.00453_f32],
  [0.58336_f32, 0.04931_f32, 0.00486_f32],
  [0.57103_f32, 0.04474_f32, 0.00529_f32],
  [0.55852_f32, 0.04028_f32, 0.00579_f32],
  [0.54583_f32, 0.03593_f32, 0.00638_f32],
  [0.53295_f32, 0.03169_f32, 0.00705_f32],
  [0.51989_f32, 0.02756_f32, 0.00780_f32],
  [0.50664_f32, 0.02354_f32, 0.00863_f32],
  [0.49321_f32, 0.01963_f32, 0.00955_f32],
  [0.47960_f32, 0.01583_f32, 0.01055_f32],
];

pub const TURBO_SRGB_BYTES: [[u8; 3]; 256] = [
  [48_u8, 18_u8, 59_u8],
  [50_u8, 21_u8, 67_u8],
  [51_u8, 24_u8, 74_u8],
  [52_u8, 27_u8, 81_u8],
  [53_u8, 30_u8, 88_u8],
  [54_u8, 33_u8, 95_u8],
  [55_u8, 36_u8, 102_u8],
  [56_u8, 39_u8, 109_u8],
  [57_u8, 42_u8, 115_u8],
  [58_u8, 45_u8, 121_u8],
  [59_u8, 47_u8, 128_u8],
  [60_u8, 50_u8, 134_u8],
  [61_u8, 53_u8, 139_u8],
  [62_u8, 56_u8, 145_u8],
  [63_u8, 59_u8, 151_u8],
  [63_u8, 62_u8, 156_u8],
  [64_u8, 64_u8, 162_u8],
  [65_u8, 67_u8, 167_u8],
  [65_u8, 70_u8, 172_u8],
  [66_u8, 73_u8, 177_u8],
  [66_u8, 75_u8, 181_u8],
  [67_u8, 78_u8, 186_u8],
  [68_u8, 81_u8, 191_u8],
  [68_u8, 84_u8, 195_u8],
  [68_u8, 86_u8, 199_u8],
  [69_u8, 89_u8, 203_u8],
  [69_u8, 92_u8, 207_u8],
  [69_u8, 94_u8, 211_u8],
  [70_u8, 97_u8, 214_u8],
  [70_u8, 100_u8, 218_u8],
  [70_u8, 102_u8, 221_u8],
  [70_u8, 105_u8, 224_u8],
  [70_u8, 107_u8, 227_u8],
  [71_u8, 110_u8, 230_u8],
  [71_u8, 113_u8, 233_u8],
  [71_u8, 115_u8, 235_u8],
  [71_u8, 118_u8, 238_u8],
  [71_u8, 120_u8, 240_u8],
  [71_u8, 123_u8, 242_u8],
  [70_u8, 125_u8, 244_u8],
  [70_u8, 128_u8, 246_u8],
  [70_u8, 130_u8, 248_u8],
  [70_u8, 133_u8, 250_u8],
  [70_u8, 135_u8, 251_u8],
  [69_u8, 138_u8, 252_u8],
  [69_u8, 140_u8, 253_u8],
  [68_u8, 143_u8, 254_u8],
  [67_u8, 145_u8, 254_u8],
  [66_u8, 148_u8, 255_u8],
  [65_u8, 150_u8, 255_u8],
  [64_u8, 153_u8, 255_u8],
  [62_u8, 155_u8, 254_u8],
  [61_u8, 158_u8, 254_u8],
  [59_u8, 160_u8, 253_u8],
  [58_u8, 163_u8, 252_u8],
  [56_u8, 165_u8, 251_u8],
  [55_u8, 168_u8, 250_u8],
  [53_u8, 171_u8, 248_u8],
  [51_u8, 173_u8, 247_u8],
  [49_u8, 175_u8, 245_u8],
  [47_u8, 178_u8, 244_u8],
  [46_u8, 180_u8, 242_u8],
  [44_u8, 183_u8, 240_u8],
  [42_u8, 185_u8, 238_u8],
  [40_u8, 188_u8, 235_u8],
  [39_u8, 190_u8, 233_u8],
  [37_u8, 192_u8, 231_u8],
  [35_u8, 195_u8, 228_u8],
  [34_u8, 197_u8, 226_u8],
  [32_u8, 199_u8, 223_u8],
  [31_u8, 201_u8, 221_u8],
  [30_u8, 203_u8, 218_u8],
  [28_u8, 205_u8, 216_u8],
  [27_u8, 208_u8, 213_u8],
  [26_u8, 210_u8, 210_u8],
  [26_u8, 212_u8, 208_u8],
  [25_u8, 213_u8, 205_u8],
  [24_u8, 215_u8, 202_u8],
  [24_u8, 217_u8, 200_u8],
  [24_u8, 219_u8, 197_u8],
  [24_u8, 221_u8, 194_u8],
  [24_u8, 222_u8, 192_u8],
  [24_u8, 224_u8, 189_u8],
  [25_u8, 226_u8, 187_u8],
  [25_u8, 227_u8, 185_u8],
  [26_u8, 228_u8, 182_u8],
  [28_u8, 230_u8, 180_u8],
  [29_u8, 231_u8, 178_u8],
  [31_u8, 233_u8, 175_u8],
  [32_u8, 234_u8, 172_u8],
  [34_u8, 235_u8, 170_u8],
  [37_u8, 236_u8, 167_u8],
  [39_u8, 238_u8, 164_u8],
  [42_u8, 239_u8, 161_u8],
  [44_u8, 240_u8, 158_u8],
  [47_u8, 241_u8, 155_u8],
  [50_u8, 242_u8, 152_u8],
  [53_u8, 243_u8, 148_u8],
  [56_u8, 244_u8, 145_u8],
  [60_u8, 245_u8, 142_u8],
  [63_u8, 246_u8, 138_u8],
  [67_u8, 247_u8, 135_u8],
  [70_u8, 248_u8, 132_u8],
  [74_u8, 248_u8, 128_u8],
  [78_u8, 249_u8, 125_u8],
  [82_u8, 250_u8, 122_u8],
  [85_u8, 250_u8, 118_u8],
  [89_u8, 251_u8, 115_u8],
  [93_u8, 252_u8, 111_u8],
  [97_u8, 252_u8, 108_u8],
  [101_u8, 253_u8, 105_u8],
  [105_u8, 253_u8, 102_u8],
  [109_u8, 254_u8, 98_u8],
  [113_u8, 254_u8, 95_u8],
  [117_u8, 254_u8, 92_u8],
  [121_u8, 254_u8, 89_u8],
  [125_u8, 255_u8, 86_u8],
  [128_u8, 255_u8, 83_u8],
  [132_u8, 255_u8, 81_u8],
  [136_u8, 255_u8, 78_u8],
  [139_u8, 255_u8, 75_u8],
  [143_u8, 255_u8, 73_u8],
  [146_u8, 255_u8, 71_u8],
  [150_u8, 254_u8, 68_u8],
  [153_u8, 254_u8, 66_u8],
  [156_u8, 254_u8, 64_u8],
  [159_u8, 253_u8, 63_u8],
  [161_u8, 253_u8, 61_u8],
  [164_u8, 252_u8, 60_u8],
  [167_u8, 252_u8, 58_u8],
  [169_u8, 251_u8, 57_u8],
  [172_u8, 251_u8, 56_u8],
  [175_u8, 250_u8, 55_u8],
  [177_u8, 249_u8, 54_u8],
  [180_u8, 248_u8, 54_u8],
  [183_u8, 247_u8, 53_u8],
  [185_u8, 246_u8, 53_u8],
  [188_u8, 245_u8, 52_u8],
  [190_u8, 244_u8, 52_u8],
  [193_u8, 243_u8, 52_u8],
  [195_u8, 241_u8, 52_u8],
  [198_u8, 240_u8, 52_u8],
  [200_u8, 239_u8, 52_u8],
  [203_u8, 237_u8, 52_u8],
  [205_u8, 236_u8, 52_u8],
  [208_u8, 234_u8, 52_u8],
  [210_u8, 233_u8, 53_u8],
  [212_u8, 231_u8, 53_u8],
  [215_u8, 229_u8, 53_u8],
  [217_u8, 228_u8, 54_u8],
  [219_u8, 226_u8, 54_u8],
  [221_u8, 224_u8, 55_u8],
  [223_u8, 223_u8, 55_u8],
  [225_u8, 221_u8, 55_u8],
  [227_u8, 219_u8, 56_u8],
  [229_u8, 217_u8, 56_u8],
  [231_u8, 215_u8, 57_u8],
  [233_u8, 213_u8, 57_u8],
  [235_u8, 211_u8, 57_u8],
  [236_u8, 209_u8, 58_u8],
  [238_u8, 207_u8, 58_u8],
  [239_u8, 205_u8, 58_u8],
  [241_u8, 203_u8, 58_u8],
  [242_u8, 201_u8, 58_u8],
  [244_u8, 199_u8, 58_u8],
  [245_u8, 197_u8, 58_u8],
  [246_u8, 195_u8, 58_u8],
  [247_u8, 193_u8, 58_u8],
  [248_u8, 190_u8, 57_u8],
  [249_u8, 188_u8, 57_u8],
  [250_u8, 186_u8, 57_u8],
  [251_u8, 184_u8, 56_u8],
  [251_u8, 182_u8, 55_u8],
  [252_u8, 179_u8, 54_u8],
  [252_u8, 177_u8, 54_u8],
  [253_u8, 174_u8, 53_u8],
  [253_u8, 172_u8, 52_u8],
  [254_u8, 169_u8, 51_u8],
  [254_u8, 167_u8, 50_u8],
  [254_u8, 164_u8, 49_u8],
  [254_u8, 161_u8, 48_u8],
  [254_u8, 158_u8, 47_u8],
  [254_u8, 155_u8, 45_u8],
  [254_u8, 153_u8, 44_u8],
  [254_u8, 150_u8, 43_u8],
  [254_u8, 147_u8, 42_u8],
  [254_u8, 144_u8, 41_u8],
  [253_u8, 141_u8, 39_u8],
  [253_u8, 138_u8, 38_u8],
  [252_u8, 135_u8, 37_u8],
  [252_u8, 132_u8, 35_u8],
  [251_u8, 129_u8, 34_u8],
  [251_u8, 126_u8, 33_u8],
  [250_u8, 123_u8, 31_u8],
  [249_u8, 120_u8, 30_u8],
  [249_u8, 117_u8, 29_u8],
  [248_u8, 114_u8, 28_u8],
  [247_u8, 111_u8, 26_u8],
  [246_u8, 108_u8, 25_u8],
  [245_u8, 105_u8, 24_u8],
  [244_u8, 102_u8, 23_u8],
  [243_u8, 99_u8, 21_u8],
  [242_u8, 96_u8, 20_u8],
  [241_u8, 93_u8, 19_u8],
  [240_u8, 91_u8, 18_u8],
  [239_u8, 88_u8, 17_u8],
  [237_u8, 85_u8, 16_u8],
  [236_u8, 83_u8, 15_u8],
  [235_u8, 80_u8, 14_u8],
  [234_u8, 78_u8, 13_u8],
  [232_u8, 75_u8, 12_u8],
  [231_u8, 73_u8, 12_u8],
  [229_u8, 71_u8, 11_u8],
  [228_u8, 69_u8, 10_u8],
  [226_u8, 67_u8, 10_u8],
  [225_u8, 65_u8, 9_u8],
  [223_u8, 63_u8, 8_u8],
  [221_u8, 61_u8, 8_u8],
  [220_u8, 59_u8, 7_u8],
  [218_u8, 57_u8, 7_u8],
  [216_u8, 55_u8, 6_u8],
  [214_u8, 53_u8, 6_u8],
  [212_u8, 51_u8, 5_u8],
  [210_u8, 49_u8, 5_u8],
  [208_u8, 47_u8, 5_u8],
  [206_u8, 45_u8, 4_u8],
  [204_u8, 43_u8, 4_u8],
  [202_u8, 42_u8, 4_u8],
  [200_u8, 40_u8, 3_u8],
  [197_u8, 38_u8, 3_u8],
  [195_u8, 37_u8, 3_u8],
  [193_u8, 35_u8, 2_u8],
  [190_u8, 33_u8, 2_u8],
  [188_u8, 32_u8, 2_u8],
  [185_u8, 30_u8, 2_u8],
  [183_u8, 29_u8, 2_u8],
  [180_u8, 27_u8, 1_u8],
  [178_u8, 26_u8, 1_u8],
  [175_u8, 24_u8, 1_u8],
  [172_u8, 23_u8, 1_u8],
  [169_u8, 22_u8, 1_u8],
  [167_u8, 20_u8, 1_u8],
  [164_u8, 19_u8, 1_u8],
  [161_u8, 18_u8, 1_u8],
  [158_u8, 16_u8, 1_u8],
  [155_u8, 15_u8, 1_u8],
  [152_u8, 14_u8, 1_u8],
  [149_u8, 13_u8, 1_u8],
  [146_u8, 11_u8, 1_u8],
  [142_u8, 10_u8, 1_u8],
  [139_u8, 9_u8, 2_u8],
  [136_u8, 8_u8, 2_u8],
  [133_u8, 7_u8, 2_u8],
  [129_u8, 6_u8, 2_u8],
  [126_u8, 5_u8, 2_u8],
  [122_u8, 4_u8, 3_u8],
];

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
