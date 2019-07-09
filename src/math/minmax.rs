pub trait MinMax {
  type Output;
  const MIN: Self::Output;
  const MAX: Self::Output;

  fn min(a: Self::Output, b: Self::Output) -> Self::Output;
  fn max(a: Self::Output, b: Self::Output) -> Self::Output;
}

macro_rules! impl_minmax {
    ($($t:ident),*) => {
        $(
            impl MinMax for $t {
                type Output = $t;

                const MIN : Self::Output = std::$t::MIN;
                const MAX : Self::Output = std::$t::MAX;

                  fn min(a: Self::Output, b: Self::Output) -> Self::Output {
                      a.min(b)
                  }
                    fn max(a: Self::Output, b: Self::Output) -> Self::Output {
                        a.max(b)
                    }


            }
        )*
    };
}

impl_minmax!(i8, u8, i16, u16, i32, u32, i64, u64, f32, f64, usize, isize);
