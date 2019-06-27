pub fn saturate(x: f32) -> f32 {
  (x.min(1_f32)).max(0_f32)
}

pub fn clamp(minval: f32, x: f32, maxval: f32) -> f32 {
  minval.max(x).min(maxval)
}