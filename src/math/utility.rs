#![allow(dead_code)]

pub fn saturate(x: f32) -> f32 {
  (x.min(1_f32)).max(0_f32)
}

pub fn clamp(minval: f32, x: f32, maxval: f32) -> f32 {
  minval.max(x).min(maxval)
}

pub fn roundup_next_power_of_two(x: u32) -> u32 {
  if x == 0 {
    x
  } else {
    let mut x = x;
    x -= 1;
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    x += 1;

    x
  }
}
