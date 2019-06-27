use crate::math::{
  vec2::{
    Vec2I16
  },
  rgb::{RGBAColor},
  rectangle::{RectangleF32}
};

use crate::hmi::{
  image::Image,
  base::UserFont,
};

#[derive(Copy, Clone, Debug)]
pub struct CmdScissor {
  pub x: i16,
  pub y: i16,
  pub w: u16,
  pub h: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct CmdLine {
  pub line_thickness: u16,
  pub begin: Vec2I16,
  pub end: Vec2I16,
  pub color: RGBAColor,
}

#[derive(Copy, Clone, Debug)]
pub struct CmdCurve {
  pub line_thickness: u16,
  pub begin: Vec2I16,
  pub end: Vec2I16,
  pub ctrl: [Vec2I16; 2],
  pub color: RGBAColor,
}

#[derive(Copy, Clone, Debug)]
pub struct CmdRect {
  pub rounding: u16,
  pub line_thickness: u16,
  pub x: i16,
  pub y: i16,
  pub w: u16,
  pub h: u16,
  pub color: RGBAColor,
}

#[derive(Copy, Clone, Debug)]
pub struct CmdRectFilled {
  pub rounding: u16,
  pub x: i16,
  pub y: i16,
  pub w: u16,
  pub h: u16,
  pub color: RGBAColor,
}

#[derive(Copy, Clone, Debug)]
pub struct CmdRectMulticolor {
  pub x: i16,
  pub y: i16,
  pub w: u16,
  pub h: u16,
  pub left: RGBAColor,
  pub top: RGBAColor,
  pub bottom: RGBAColor,
  pub right: RGBAColor,
}

#[derive(Copy, Clone, Debug)]
pub struct CmdTriangle {
  pub line_thickness: u16,
  pub a: Vec2I16,
  pub b: Vec2I16,
  pub c: Vec2I16,
  pub color: RGBAColor,
}

#[derive(Copy, Clone, Debug)]
pub struct CmdTriangleFilled {
  pub a: Vec2I16,
  pub b: Vec2I16,
  pub c: Vec2I16,
  pub color: RGBAColor,
}

#[derive(Copy, Clone, Debug)]
pub struct CmdCircle {
  pub x: i16,
  pub y: i16,
  pub line_thickness: u16,
  pub w: u16,
  pub h: u16,
  pub color: RGBAColor,
}

#[derive(Copy, Clone, Debug)]
pub struct CmdCircleFilled {
  pub x: i16,
  pub y: i16,
  pub w: u16,
  pub h: u16,
  pub color: RGBAColor,
}

#[derive(Copy, Clone, Debug)]
pub struct CmdArc {
  pub cx: i16,
  pub cy: i16,
  pub r: u16,
  pub line_thickness: u16,
  pub a: [f32; 2],
  pub color: RGBAColor,
}

#[derive(Copy, Clone, Debug)]
pub struct CmdArcFilled {
  pub cx: i16,
  pub cy: i16,
  pub r: u16,
  pub a: [f32; 2],
  pub color: RGBAColor,
}

#[derive(Clone, Debug)]
pub struct CmdPolygon {
  pub color: RGBAColor,
  pub line_thickness: u16,
  pub points: Vec<Vec2I16>,
}

#[derive(Clone, Debug)]
pub struct CmdPolygonFilled {
  pub color: RGBAColor,
  pub points: Vec<Vec2I16>,
}

#[derive(Clone, Debug)]
pub struct CmdPolyline {
  pub color: RGBAColor,
  pub line_thickness: u16,
  pub points: Vec<Vec2I16>,
}

#[derive(Copy, Clone, Debug)]
pub struct CmdImage {
  pub x: i16,
  pub y: i16,
  pub w: u16,
  pub h: u16,
  pub img: Image,
  pub color: RGBAColor,
}

#[derive(Clone, Debug)]
pub struct CmdText {
  pub font: UserFont,
  pub background: RGBAColor,
  pub foreground: RGBAColor,
  pub x: i16,
  pub y: i16,
  pub w: u16,
  pub h: u16,
  pub height: f32,
  pub text: String,
}

#[derive(Debug, Clone)]
pub enum Command {
  Nop,
  Scissor(CmdScissor),
  Line(CmdLine),
  Curve(CmdCurve),
  Rect(CmdRect),
  RectFilled(CmdRectFilled),
  RectMulticolor(CmdRectMulticolor),
  Triangle(CmdTriangle),
  TriangleFilled(CmdTriangleFilled),
  Circle(CmdCircle),
  CircleFilled(CmdCircleFilled),
  Arc(CmdArc),
  ArcFilled(CmdArcFilled),
  Polygon(CmdPolygon),
  PolygonFilled(CmdPolygonFilled),
  Polyline(CmdPolyline),
  Image(CmdImage),
  Text(CmdText),
}

#[derive(Clone, Debug)]
pub struct CommandBuffer {
  clip: Option<RectangleF32>,
  base: Vec<Command>,
}

impl CommandBuffer {
  pub fn new(clip: Option<RectangleF32>, min_buffer_size: usize) -> CommandBuffer {
    CommandBuffer {
      clip,
      base: Vec::with_capacity(min_buffer_size),
    }
  }

  pub fn stroke_line(
    &mut self,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    line_thickness: f32,
    color: RGBAColor,
  ) {
    let cmd = CmdLine {
      line_thickness: line_thickness as u16,
      begin: Vec2I16::new(x0 as i16, y0 as i16),
      end: Vec2I16::new(x1 as i16, y1 as i16),
      color,
    };

    self.base.push(Command::Line(cmd));
  }

  pub fn stroke_curve(
    &mut self,
    ax: f32,
    ay: f32,
    ctrl0x: f32,
    ctrl0y: f32,
    ctrl1x: f32,
    ctrl1y: f32,
    bx: f32,
    by: f32,
    line_thickness: f32,
    color: RGBAColor,
  ) {
    let cmd = CmdCurve {
      line_thickness: line_thickness as u16,
      begin: Vec2I16::new(ax as i16, ay as i16),
      end: Vec2I16::new(bx as i16, by as i16),
      ctrl: [
        Vec2I16::new(ctrl0x as i16, ctrl0y as i16),
        Vec2I16::new(ctrl1x as i16, ctrl1y as i16),
      ],
      color,
    };

    self.base.push(Command::Curve(cmd));
  }

  pub fn stroke_rect(
    &mut self,
    rect: RectangleF32,
    rounding: f32,
    line_thickness: f32,
    color: RGBAColor,
  ) {
    if color.a == 0 || rect.w == 0_f32 || rect.h == 0_f32 || line_thickness <= 0_f32 {
      return;
    }

    let clipped = self
      .clip
      .map_or(false, |clip_rect| !clip_rect.intersect(&rect));
    if clipped {
      return;
    }

    let cmd = CmdRect {
      rounding: rounding as u16,
      line_thickness: line_thickness as u16,
      x: rect.x as i16,
      y: rect.y as i16,
      w: rect.w as u16,
      h: rect.h as u16,
      color,
    };

    self.base.push(Command::Rect(cmd));
  }

  pub fn stroke_circle(&mut self, r: RectangleF32, line_thickness: f32, color: RGBAColor) {
    if r.w == 0_f32 || r.h == 0_f32 || line_thickness <= 0_f32 {
      return;
    }

    let is_clipped = self.clip.map_or(false, |clip_r| !clip_r.intersect(&r));
    if is_clipped {
      return;
    }

    let cmd = CmdCircle {
      x: r.x as i16,
      y: r.y as i16,
      line_thickness: line_thickness as u16,
      w: r.w.max(0_f32) as u16,
      h: r.h.max(0_f32) as u16,
      color,
    };

    self.base.push(Command::Circle(cmd));
  }

  pub fn stroke_arc(
    &mut self,
    cx: f32,
    cy: f32,
    radius: f32,
    a_min: f32,
    a_max: f32,
    line_thickness: f32,
    color: RGBAColor,
  ) {
    if color.a == 0 || line_thickness <= 0_f32 {
      return;
    }

    let cmd = CmdArc {
      cx: cx as i16,
      cy: cy as i16,
      r: radius as u16,
      line_thickness: line_thickness as u16,
      a: [a_min, a_max],
      color,
    };

    self.base.push(Command::Arc(cmd));
  }

  pub fn stroke_triangle(
    &mut self,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    line_thickness: f32,
    color: RGBAColor,
  ) {
    if color.a == 0 || line_thickness <= 0_f32 {
      return;
    }

    let is_clipped = self.clip.map_or(false, |clip_r| {
      !clip_r.contains_point(x0, y0)
        && !clip_r.contains_point(x1, y1)
        && !clip_r.contains_point(x2, y2)
    });
    if is_clipped {
      return;
    }

    let cmd = CmdTriangle {
      line_thickness: line_thickness as u16,
      a: Vec2I16::new(x0 as i16, y0 as i16),
      b: Vec2I16::new(x1 as i16, y1 as i16),
      c: Vec2I16::new(x2 as i16, y2 as i16),
      color,
    };

    self.base.push(Command::Triangle(cmd));
  }

  pub fn stroke_polyline(&mut self, points: &[f32], line_thickness: f32, color: RGBAColor) {
    if color.a == 0 || line_thickness <= 0_f32 {
      return;
    }

    let cmd = CmdPolyline {
      color,
      line_thickness: line_thickness as u16,
      points: points
        .iter()
        .step_by(2)
        .zip(points.iter().skip(1).step_by(2))
        .map(|(&x, &y)| Vec2I16::new(x as i16, y as i16))
        .collect(),
    };

    self.base.push(Command::Polyline(cmd));
  }

  pub fn stroke_polygon(&mut self, points: &[f32], line_thickness: f32, color: RGBAColor) {
    if color.a == 0 || line_thickness <= 0_f32 {
      return;
    }

    let cmd = CmdPolygon {
      color,
      line_thickness: line_thickness as u16,
      points: points
        .iter()
        .step_by(2)
        .zip(points.iter().skip(1).step_by(2))
        .map(|(&x, &y)| Vec2I16::new(x as i16, y as i16))
        .collect(),
    };

    self.base.push(Command::Polygon(cmd));
  }

  pub fn fill_rect(&mut self, rect: RectangleF32, rounding: f32, color: RGBAColor) {
    if color.a == 0 || rect.w == 0_f32 || rect.h == 0_f32 {
      return;
    }

    let is_clipped = self.clip.map_or(false, |clip_r| !clip_r.intersect(&rect));
    if is_clipped {
      return;
    }
    let cmd = CmdRectFilled {
      rounding: rounding as u16,
      x: rect.x as i16,
      y: rect.y as i16,
      w: rect.w as u16,
      h: rect.h as u16,
      color,
    };

    self.base.push(Command::RectFilled(cmd));
  }

  pub fn fill_rect_multicolor(
    &mut self,
    rect: RectangleF32,
    left: RGBAColor,
    top: RGBAColor,
    right: RGBAColor,
    bottom: RGBAColor,
  ) {
    if rect.w == 0_f32 || rect.h == 0_f32 {
      return;
    }

    let is_clipped = self.clip.map_or(false, |clip_r| !clip_r.intersect(&rect));
    if is_clipped {
      return;
    }

    let cmd = CmdRectMulticolor {
      x: rect.x as i16,
      y: rect.y as i16,
      w: rect.w as u16,
      h: rect.h as u16,
      left,
      top,
      bottom,
      right,
    };

    self.base.push(Command::RectMulticolor(cmd));
  }

  pub fn fill_circle(&mut self, r: RectangleF32, color: RGBAColor) {
    if color.a == 0 || r.w == 0_f32 || r.h == 0_f32 {
      return;
    }

    let is_clipped = self.clip.map_or(false, |clip_r| !clip_r.intersect(&r));
    if is_clipped {
      return;
    }

    let cmd = CmdCircleFilled {
      x: r.x as i16,
      y: r.y as i16,
      w: r.w as u16,
      h: r.h as u16,
      color,
    };

    self.base.push(Command::CircleFilled(cmd));
  }

  pub fn fill_arc(
    &mut self,
    cx: f32,
    cy: f32,
    radius: f32,
    a_min: f32,
    a_max: f32,
    color: RGBAColor,
  ) {
    if color.a == 0 {
      return;
    }
    let cmd = CmdArcFilled {
      cx: cx as i16,
      cy: cy as i16,
      r: radius as u16,
      a: [a_min, a_max],
      color,
    };

    self.base.push(Command::ArcFilled(cmd));
  }

  pub fn fill_triangle(
    &mut self,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    color: RGBAColor,
  ) {
    if color.a == 0 {
      return;
    }

    let is_clipped = self.clip.map_or(false, |clip_r| {
      !clip_r.contains_point(x0, y0)
        && !clip_r.contains_point(x1, y1)
        && !clip_r.contains_point(x2, y1)
    });
    if is_clipped {
      return;
    }

    let cmd = CmdTriangleFilled {
      a: Vec2I16::new(x0 as i16, y0 as i16),
      b: Vec2I16::new(x1 as i16, y1 as i16),
      c: Vec2I16::new(x2 as i16, y2 as i16),
      color,
    };

    self.base.push(Command::TriangleFilled(cmd));
  }

  pub fn fill_polygon(&mut self, points: &[f32], color: RGBAColor) {
    if color.a == 0 {
      return;
    }

    let cmd = CmdPolygonFilled {
      color,
      points: points
        .iter()
        .step_by(2)
        .zip(points.iter().skip(1).step_by(2))
        .map(|(&x, &y)| Vec2I16::new(x as i16, y as i16))
        .collect(),
    };

    self.base.push(Command::PolygonFilled(cmd));
  }

  pub fn draw_image(&mut self, r: RectangleF32, img: Image, color: RGBAColor) {
    let is_clipped = self.clip.map_or(false, |clip_r| {
      clip_r.w == 0_f32 || clip_r.h == 0_f32 || !clip_r.intersect(&r)
    });
    if is_clipped {
      return;
    }

    let cmd = CmdImage {
      x: r.x as i16,
      y: r.y as i16,
      w: r.w as u16,
      h: r.h as u16,
      img,
      color,
    };

    self.base.push(Command::Image(cmd));
  }

  pub fn draw_text(
    &mut self,
    _r: RectangleF32,
    _s: &str,
    _font: UserFont,
    _background: RGBAColor,
    _foreground: RGBAColor,
  ) {

  }

  pub fn push_scissor(&mut self, r: RectangleF32) {
    self.clip.replace(r);

    let cmd = CmdScissor {
      x: r.x as i16,
      y: r.y as i16,
      w: r.w.max(0_f32) as u16,
      h: r.h.max(0_f32) as u16,
    };
    self.base.push(Command::Scissor(cmd));
  }
}

#[cfg(feature = "VERTEX_BUFFER_OUTPUT")]
mod vertex_buffer_output {}
