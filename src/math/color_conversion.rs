use crate::math::colors::{HslColor, HsvColor, RGBAColor, RGBAColorF32, XyzColor};

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

impl std::convert::From<HsvColor> for RGBAColorF32 {
    fn from(hsv : HsvColor) -> RGBAColorF32 {
        let h = hsv.h / 360f32;
        let s = hsv.s * 0.01f32;
        let v = hsv.v * 0.01f32;

        let i = (h * 6f32).floor();
        let f = h * 6f32 - i;
        let p = v * (1f32 - s);
        let q = v * (1f32 -f * s);
        let t = v * (1f32 - (1f32 - f) * s);

        match (i as i32 % 6) {
            0 => RGBAColorF32::new(v, t, p),
            1 => RGBAColorF32::new(q, v, p),
            2 => RGBAColorF32::new(p, v, t),
            3 => RGBAColorF32::new(p, q, v),
            4 => RGBAColorF32::new(t, p, v),
            5 => RGBAColorF32::new(v, p, q),
            _ => panic!("Whoaa there buddy! Nice color!")
        }
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