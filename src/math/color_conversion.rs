use crate::math::{
  colors::{HslColor, HsvColor, RGBAColorF32, XyzColor},
  utility::clamp,
};

impl std::convert::From<HslColor> for RGBAColorF32 {
  fn from(hsl: HslColor) -> RGBAColorF32 {
    let h = hsl.h;
    let s = hsl.s;
    let l = hsl.l;

    let a = s * l.min(1_f32 - l);

    let f = |n: f32| {
      let k = (n + h / 30_f32) % 12_f32;
      l - a * (-1_f32).max(1_f32.min((9_f32 - k).min(k - 3_f32)))
    };

    RGBAColorF32::new(f(0_f32), f(8_f32), f(4_f32))
  }
}

impl std::convert::From<RGBAColorF32> for HslColor {
  fn from(rgb: RGBAColorF32) -> HslColor {
    let cmax = rgb.r.max(rgb.g.max(rgb.b));
    let cmin = rgb.r.min(rgb.g.min(rgb.b));
    let delta = cmax - cmin;

    const EPSILON: f32 = 0.00001f32;

    let is_zero = |a: f32| a.abs() < EPSILON;

    let approx_eq = |a: f32, b: f32| is_zero(a - b);

    let mut h = if delta.abs() < 0.0004f32 {
      0f32
    } else if approx_eq(cmax, rgb.r) {
      60f32 * (((rgb.g - rgb.b) / delta) % 6f32)
    } else if approx_eq(cmax, rgb.g) {
      60f32 * ((rgb.b - rgb.r) / delta + 2f32)
    } else if approx_eq(cmax, rgb.b) {
      60f32 * ((rgb.r - rgb.g) / delta + 4f32)
    } else {
      0f32
    };

    if h < 0f32 {
      h += 360f32;
    }

    let l = (cmax + cmin) * 0.5f32;
    let s = if is_zero(delta) {
      0f32
    } else {
      delta / (1f32 - (2f32 * l - 1f32).abs())
    };

    HslColor::new(h, s, l)
  }
}

impl std::convert::From<HsvColor> for RGBAColorF32 {
  fn from(hsv: HsvColor) -> RGBAColorF32 {
    let h = hsv.h / 360f32;
    let s = hsv.s * 0.01f32;
    let v = hsv.v * 0.01f32;

    let i = (h * 6f32).floor();
    let f = h * 6f32 - i;
    let p = v * (1f32 - s);
    let q = v * (1f32 - f * s);
    let t = v * (1f32 - (1f32 - f) * s);

    match i as i32 % 6 {
      0 => RGBAColorF32::new(v, t, p),
      1 => RGBAColorF32::new(q, v, p),
      2 => RGBAColorF32::new(p, v, t),
      3 => RGBAColorF32::new(p, q, v),
      4 => RGBAColorF32::new(t, p, v),
      5 => RGBAColorF32::new(v, p, q),
      _ => panic!("Whoaa there buddy! Nice color!"),
    }
  }
}

impl std::convert::From<RGBAColorF32> for HsvColor {
  fn from(rgb: RGBAColorF32) -> HsvColor {
    let max = rgb.r.max(rgb.g.max(rgb.b));
    let min = rgb.r.min(rgb.g.min(rgb.b));

    let v = max;
    let s;
    let mut h;

    if max != 0f32 {
      s = (max - min) / max;
    } else {
      s = 0f32;
      h = std::f32::MAX;
      return HsvColor::new(h, s, v);
    }

    let delta = max - min;

    if max == rgb.r {
      h = (rgb.g - rgb.b) / delta;
    } else if rgb.g == max {
      h = 2f32 + (rgb.b - rgb.r) / delta;
    } else {
      h = 4f32 + (rgb.r - rgb.g) / delta;
    }

    h *= 60f32;
    if h < 0f32 {
      h += 360f32;
    }

    HsvColor::new(h, s, v)
  }
}

impl std::convert::From<XyzColor> for RGBAColorF32 {
  fn from(xyz: XyzColor) -> RGBAColorF32 {
    let x = xyz.x;
    let y = xyz.y;
    let z = xyz.z;

    let r_linear =
      clamp(0f32, 3.2406f32 * x - 1.5372f32 * y - 0.4986f32 * z, 1f32);
    let g_linear =
      clamp(0f32, -0.9689f32 * x + 1.8758f32 * y + 0.0415f32 * z, 1f32);
    let b_linear =
      clamp(0f32, 0.0557f32 * x - 0.2040f32 * y + 1.0570f32 * z, 1f32);

    let correction_fn = |clr_val: f32| {
      let a = 0.055f32;

      if clr_val <= 0.0031308f32 {
        12.92f32 * clr_val
      } else {
        (1f32 + a) * clr_val.powf(1f32 / 2.4f32) - a
      }
    };

    RGBAColorF32::new(
      correction_fn(r_linear),
      correction_fn(g_linear),
      correction_fn(b_linear),
    )
  }
}

impl std::convert::From<RGBAColorF32> for XyzColor {
  fn from(rgb: RGBAColorF32) -> XyzColor {
    let correct_color_fn = |clr: f32| {
      if clr <= 0.04045f32 {
        clr / 12.92f32
      } else {
        let constant_val = 0.055f32;
        ((clr + constant_val) / (1f32 + constant_val)).powf(2.4f32)
      }
    };

    let r = correct_color_fn(rgb.r);
    let g = correct_color_fn(rgb.g);
    let b = correct_color_fn(rgb.b);

    XyzColor::new(
      0.4124f32 * r + 0.3576f32 * g + 0.1805f32 * b,
      0.2126f32 * r + 0.7152f32 * g + 0.0722f32 * b,
      0.0193f32 * r + 0.1192f32 * g + 0.9505f32 * b,
    )
  }
}

#[cfg(test)]
mod tests {
  use crate::math::color_conversion::*;

  #[test]
  fn test_hsl_to_rgb() {
    assert_eq!(
      RGBAColorF32::from(HslColor::new(360_f32, 0.7_f32, 0.5_f32)),
      RGBAColorF32::new(0.85f32, 0.15f32, 0.15f32)
    );
  }
}
