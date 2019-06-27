use crate::math::{
  rectangle::RectangleF32,
  rgb::{RGBAColor, RGBAColorF32},
  vec2::{normalize, Vec2F32},
  vertex_types::VertexPTC,
};

use crate::hmi::{
  base::{AntialiasingType, ConvertConfig, GenericHandle, UserFont},
  commands::Command,
  image::Image,
};

pub type DrawIndexType = u16;

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum DrawListStroke {
  Open,
  Closed,
}

#[derive(Copy, Debug, Clone)]
pub struct DrawCommand {
  pub element_count: u32,
  pub clip_rect: RectangleF32,
  pub texture: GenericHandle,
}

#[derive(Debug)]
pub struct DrawList<'a> {
  clip_rect: RectangleF32,
  circle_vtx: Vec<Vec2F32>,
  config: ConvertConfig,
  buffer: &'a mut Vec<DrawCommand>,
  vertices: &'a mut Vec<VertexPTC>,
  elements: &'a mut Vec<DrawIndexType>,
  path: std::cell::RefCell<Vec<Vec2F32>>,
  line_aa: AntialiasingType,
  shape_aa: AntialiasingType,
}

impl<'a> DrawList<'a> {
  fn null_rectangle() -> RectangleF32 {
    RectangleF32::new(-8192_f32, -8192_f32, 16834_f32, 16834_f32)
  }

  pub fn new(
    config: ConvertConfig,
    cmds: &'a mut Vec<DrawCommand>,
    vertices: &'a mut Vec<VertexPTC>,
    elements: &'a mut Vec<DrawIndexType>,
    line_aa: AntialiasingType,
    shape_aa: AntialiasingType,
  ) -> Self {
    const GEN_CIRCLE_VERTICES_COUNT: i32 = 12;

    DrawList {
      clip_rect: Self::null_rectangle(),
      circle_vtx: (0..GEN_CIRCLE_VERTICES_COUNT)
        .map(|idx| {
          let a = idx as f32 / (GEN_CIRCLE_VERTICES_COUNT as f32 * 2_f32 * std::f32::consts::PI);
          Vec2F32::new(a.cos(), a.sin())
        })
        .collect(),
      config,
      buffer: cmds,
      vertices,
      elements,
      path: std::cell::RefCell::new(vec![]),
      line_aa,
      shape_aa,
    }
  }

  fn path_last(&self) -> Vec2F32 {
    if self.path.borrow().is_empty() {
      panic!("Path is empty!");
    }

    *self.path.borrow().last().unwrap()
  }

  fn push_command(&mut self, clip: RectangleF32, texture: GenericHandle) {
    let cmd = DrawCommand {
      element_count: 0,
      clip_rect: clip,
      texture,
    };

    self.buffer.push(cmd);
    self.clip_rect = clip;
  }

  fn add_clip(&mut self, rect: RectangleF32) {
    if self.buffer.is_empty() {
      self.push_command(rect, self.config.null.texture);
      return;
    }

    let prev_cmd_texture = self
      .buffer
      .last_mut()
      .and_then(|last_cmd| {
        if last_cmd.element_count == 0 {
          last_cmd.clip_rect = rect;
        }

        Some(last_cmd.texture)
      })
      .unwrap();

    self.push_command(rect, prev_cmd_texture);
  }

  fn push_image(&mut self, texture: GenericHandle) {
    if self.buffer.is_empty() {
      self.push_command(Self::null_rectangle(), texture);
    }

    self
      .buffer
      .last_mut()
      .and_then(|prev_cmd| {
        if prev_cmd.element_count == 0 {
          prev_cmd.texture = texture;
          None
        } else if prev_cmd.texture != texture {
          Some(*prev_cmd)
        } else {
          None
        }
      })
      .and_then(|prev| {
        self.push_command(prev.clip_rect, texture);
        Some(())
      });
  }

  fn draw_vertex(
    _config: &ConvertConfig,
    pos: Vec2F32,
    uv: Vec2F32,
    color: RGBAColorF32,
  ) -> VertexPTC {
    VertexPTC {
      color,
      pos,
      texcoords: uv,
    }
  }

  pub fn stroke_poly_line(
    &mut self,
    points: &[Vec2F32],
    color: RGBAColor,
    path_type: DrawListStroke,
    thickness: f32,
    _aliasing: AntialiasingType,
  ) {
    if points.len() < 2 {
      return;
    }

    let color = RGBAColor::new(
      color.r,
      color.g,
      color.b,
      (color.a as f32 * self.config.global_alpha) as u8,
    );

    let count = if path_type == DrawListStroke::Open {
      points.len() - 1
    } else {
      points.len()
    };

    let thick_line = thickness > 1_f32;
    let col = RGBAColorF32::from(color);
    let col_trans = RGBAColorF32::new(col.r, col.g, col.b, 0_f32);

    //
    // aliased only for now

    // let vtx_count = count * 4;

    (0..count).for_each(|i1| {
      let uv = self.config.null.uv;
      let i2 = if (i1 + 1) == points.len() { 0 } else { i1 + 1 };

      let p1 = points[i1];
      let p2 = points[i2];

      let (dx, dy) = (normalize(p2 - p1) * thickness * 0.5_f32).into();

      // let diff = normalize(p2 - p1);
      // let dx = diff.x * (thickness * 0.5_f32);
      // let dy = diff.y * (thickness * 0.5_f32);

      let idx = self.vertices.len();

      [
        Vec2F32::new(dy, -dx) + p1,
        Vec2F32::new(dy, -dx) + p2,
        Vec2F32::new(-dy, dx) + p2,
        Vec2F32::new(-dy, dx) + p1,
      ]
      .into_iter()
      .for_each(|&pos| {
        self
          .vertices
          .push(Self::draw_vertex(&self.config, pos, uv, col));
      });

      [0, 1, 2, 0, 2, 3]
        .into_iter()
        .for_each(|&offset| self.elements.push((idx + offset) as DrawIndexType));

      //
      // update element count for the current draw command
      let element_count = self.elements.len() as u32;
      self.buffer.last_mut().and_then(|cmd| {
        cmd.element_count = element_count;
        Some(())
      });

    });
  }

  pub fn fill_poly_convex(
    &mut self,
    points: &[Vec2F32],
    color: RGBAColor,
    _aliasing: AntialiasingType,
  ) {
    if points.len() < 3 {
      return;
    }

    let col = RGBAColorF32::from(color);
    let idx = self.vertices.len();

    // let idx_count = (points.len() - 2) * 3;
    // self.buffer.last_mut().and_then(|cmd| {
    //   cmd.element_count += idx_count as u32;
    //   Some(())
    // });

    points.iter().for_each(|&vertex| {
      self.vertices.push(Self::draw_vertex(
        &self.config,
        vertex,
        self.config.null.uv,
        col,
      ));
    });

    (2..points.len()).into_iter().for_each(|offset| {
      self.elements.push(idx as DrawIndexType);
      self.elements.push((idx + offset - 1) as DrawIndexType);
      self.elements.push((idx + offset) as DrawIndexType);
    });

    let element_count = self.elements.len() as u32;
    self.buffer.last_mut().and_then(|cmd| {
      cmd.element_count = element_count;
      Some(())
    });
  }

  fn path_line_to(&mut self, pos: Vec2F32) {
    if self.buffer.is_empty() {
      self.add_clip(Self::null_rectangle());
    }

    self
      .buffer
      .last()
      .and_then(|cmd| {
        if cmd.texture != self.config.null.texture {
          Some(self.config.null.texture)
        } else {
          None
        }
      })
      .map(|img| self.push_image(img));

    self.path.borrow_mut().push(pos);
  }

  fn path_arc_to_fast(&mut self, center: Vec2F32, radius: f32, a_min: i32, a_max: i32) {
    if a_min > a_max {
      return;
    }

    (a_min..a_max + 1).into_iter().for_each(|a| {
      let c = self.circle_vtx[(a as usize) % self.circle_vtx.len()];
      self.path_line_to(center + c * radius);
    });
  }

  fn path_arc_to(&mut self, center: Vec2F32, radius: f32, a_min: f32, a_max: f32, segments: u32) {
    if radius == 0_f32 {
      return;
    }

    // This algorithm for arc drawing relies on these two trigonometric identities[1]:
    //       sin(a + b) = sin(a) * cos(b) + cos(a) * sin(b)
    //       cos(a + b) = cos(a) * cos(b) - sin(a) * sin(b)

    //   Two coordinates (x, y) of a point on a circle centered on
    //   the origin can be written in polar form as:
    //       x = r * cos(a)
    //       y = r * sin(a)
    //   where r is the radius of the circle,
    //       a is the angle between (x, y) and the origin.

    //   This allows us to rotate the coordinates around the
    //   origin by an angle b using the following transformation:
    //       x' = r * cos(a + b) = x * cos(b) - y * sin(b)
    //       y' = r * sin(a + b) = y * cos(b) + x * sin(b)

    //   [1] https://en.wikipedia.org/wiki/List_of_trigonometric_identities#Angle_sum_and_difference_identities

    let d_angle = (a_max - a_min) / segments as f32;
    let sin_d = d_angle.sin();
    let cos_d = d_angle.cos();

    let mut c = Vec2F32::new(a_min.cos() * radius, a_min.sin() * radius);
    (0..segments + 1).for_each(|_| {
      let vertex = center + c;
      self.path_line_to(vertex);

      c = Vec2F32::new(c.x * cos_d - c.y * sin_d, c.y * cos_d + c.x * sin_d);
    });
  }

  fn path_rect_to(&mut self, a: Vec2F32, b: Vec2F32, rounding: f32) {
    let r = {
      let r = rounding;
      let dist = b - a;
      let r = if dist.x < 0_f32 {
        r.min(-dist.x)
      } else {
        r.min(dist.x)
      };
      let r = if dist.y < 0_f32 {
        r.min(-dist.y)
      } else {
        r.min(dist.y)
      };

      r
    };

    if r == 0_f32 {
      self.path_line_to(a);
      self.path_line_to(Vec2F32::new(b.x, a.y));
      self.path_line_to(b);
      self.path_line_to(Vec2F32::new(a.x, b.y));
    } else {
      self.path_arc_to_fast(a + Vec2F32::same(r), r, 6, 9);
      self.path_arc_to_fast(b + Vec2F32::new(-r, r), r, 9, 12);
      self.path_arc_to_fast(b - Vec2F32::same(r), r, 0, 3);
      self.path_arc_to_fast(a + Vec2F32::new(r, -r), r, 3, 6);
    }
  }

  fn path_curve_to(&mut self, p2: Vec2F32, p3: Vec2F32, p4: Vec2F32, segments: u32) {
    if self.path.borrow().is_empty() {
      return;
    }

    let segments = segments.max(1);
    let p1 = *self.path.borrow().last().unwrap();
    let t_step = 1_f32 / segments as f32;

    (1..segments + 1).for_each(|i_step| {
      let t = t_step * i_step as f32;
      let u = 1_f32 - t;
      let w1 = u * u * u;
      let w2 = 3_f32 * u * u * t;
      let w3 = 3_f32 * u * t * t;
      let w4 = t * t * t;

      let vertex = p1 * w1 + p2 * w2 + p3 * w3 + p4 * w4;
      self.path_line_to(vertex);
    });
  }

  fn path_fill(&mut self, color: RGBAColor) {
    let path = self.path.replace(vec![]);
    self.fill_poly_convex(&path, color, self.config.shape_aa);
  }

  fn path_stroke(&mut self, color: RGBAColor, path_type: DrawListStroke, thickness: f32) {
    let path = self.path.replace(vec![]);
    self.stroke_poly_line(&path, color, path_type, thickness, self.config.line_aa);
  }

  fn stroke_line(&mut self, a: Vec2F32, b: Vec2F32, col: RGBAColor, thickness: f32) {
    if col.a == 0 {
      return;
    }

    if self.line_aa == AntialiasingType::On {
      self.path_line_to(a);
      self.path_line_to(b);
    } else {
      self.path_line_to(a - Vec2F32::same(0.5_f32));
      self.path_line_to(b - Vec2F32::same(0.5_f32));
    }

    self.path_stroke(col, DrawListStroke::Open, thickness);
  }

  fn fill_rect(&mut self, rect: RectangleF32, col: RGBAColor, rounding: f32) {
    if col.a == 0 {
      return;
    }

    if self.line_aa == AntialiasingType::On {
      self.path_rect_to(
        Vec2F32::new(rect.x, rect.y),
        Vec2F32::new(rect.x + rect.w, rect.y + rect.h),
        rounding,
      );
    } else {
      self.path_rect_to(
        Vec2F32::new(rect.x - 0.5_f32, rect.y - 0.5_f32),
        Vec2F32::new(rect.x + rect.w, rect.y + rect.h),
        rounding,
      );
    }
    self.path_fill(col);
  }

  fn stroke_rect(&mut self, rect: RectangleF32, col: RGBAColor, rounding: f32, thickness: f32) {
    if col.a == 0 {
      return;
    }

    if self.line_aa == AntialiasingType::On {
      self.path_rect_to(
        Vec2F32::new(rect.x, rect.y),
        Vec2F32::new(rect.x + rect.w, rect.y + rect.h),
        rounding,
      );
    } else {
      self.path_rect_to(
        Vec2F32::new(rect.x - 0.5_f32, rect.y - 0.5_f32),
        Vec2F32::new(rect.x + rect.w, rect.y + rect.h),
        rounding,
      );
    }

    self.path_stroke(col, DrawListStroke::Closed, thickness);
  }

  fn fill_rect_multi_color(
    &mut self,
    rect: RectangleF32,
    left: RGBAColor,
    top: RGBAColor,
    right: RGBAColor,
    bottom: RGBAColor,
  ) {
    let col_left = RGBAColorF32::from(left);
    let col_right = RGBAColorF32::from(right);
    let col_top = RGBAColorF32::from(top);
    let col_bottom = RGBAColorF32::from(bottom);

    self.push_image(self.config.null.texture);
    let idx = self.vertices.len() as DrawIndexType;

    let vertices = [
      (Vec2F32::new(rect.x, rect.y), col_left),
      (Vec2F32::new(rect.x + rect.w, rect.y), col_top),
      (Vec2F32::new(rect.x + rect.w, rect.y + rect.h), col_right),
      (Vec2F32::new(rect.x, rect.y + rect.h), col_bottom),
    ];

    vertices.into_iter().for_each(|&(pos, col)| {
      self.vertices.push(Self::draw_vertex(
        &self.config,
        pos,
        self.config.null.uv,
        col,
      ));
    });

    [0, 1, 2, 0, 2, 3]
      .into_iter()
      .for_each(|&offset| self.elements.push(idx + offset as DrawIndexType));

    let element_count = self.elements.len() as u32;
    self.buffer.last_mut().and_then(|cmd| {
      cmd.element_count = element_count;
      Some(())
    });
  }

  fn stroke_triangle(
    &mut self,
    a: Vec2F32,
    b: Vec2F32,
    c: Vec2F32,
    col: RGBAColor,
    thickness: f32,
  ) {
    if col.a == 0 {
      return;
    }

    self.path_line_to(a);
    self.path_line_to(b);
    self.path_line_to(c);
    self.path_stroke(col, DrawListStroke::Closed, thickness);
  }

  fn fill_triangle(&mut self, a: Vec2F32, b: Vec2F32, c: Vec2F32, col: RGBAColor) {
    if col.a == 0 {
      return;
    }

    self.path_line_to(a);
    self.path_line_to(b);
    self.path_line_to(c);
    self.path_fill(col);
  }

  fn fill_circle(&mut self, center: Vec2F32, radius: f32, col: RGBAColor, segments: u32) {
    if col.a == 0 {
      return;
    }

    let a_max = std::f32::consts::PI * 2_f32 * ((segments as f32 - 1_f32) / segments as f32);
    self.path_arc_to(center, radius, 0_f32, a_max, segments);
    self.path_fill(col);
  }

  fn stroke_circle(
    &mut self,
    center: Vec2F32,
    radius: f32,
    col: RGBAColor,
    segments: u32,
    thickness: f32,
  ) {
    if col.a == 0 {
      return;
    }

    let a_max = std::f32::consts::PI * 2_f32 * ((segments as f32 - 1_f32) / segments as f32);
    self.path_arc_to(center, radius, 0_f32, a_max, segments);
    self.path_stroke(col, DrawListStroke::Closed, thickness);
  }

  fn stroke_curve(
    &mut self,
    p0: Vec2F32,
    cp0: Vec2F32,
    cp1: Vec2F32,
    p1: Vec2F32,
    col: RGBAColor,
    segments: u32,
    thickness: f32,
  ) {
    if col.a == 0 {
      return;
    }

    self.path_line_to(p0);
    self.path_curve_to(cp0, cp1, p1, segments);
    self.path_stroke(col, DrawListStroke::Open, thickness);
  }

  fn push_rect_uv(&mut self, a: Vec2F32, c: Vec2F32, uva: Vec2F32, uvc: Vec2F32, color: RGBAColor) {
    let col = RGBAColorF32::from(color);
    let uvb = Vec2F32::new(uvc.x, uva.y);
    let uvd = Vec2F32::new(uva.x, uvc.y);

    let b = Vec2F32::new(c.x, a.y);
    let d = Vec2F32::new(a.x, c.y);

    let idx = self.vertices.len() as DrawIndexType;

    [(a, uva), (b, uvb), (c, uvc), (d, uvd)]
      .into_iter()
      .for_each(|&(v, uv)| {
        self
          .vertices
          .push(Self::draw_vertex(&self.config, v, uv, col))
      });

    [0, 1, 2, 0, 2, 3]
      .into_iter()
      .for_each(|&offset| self.elements.push(offset as DrawIndexType + idx));

    let element_count = self.elements.len() as u32;
    self.buffer.last_mut().and_then(|cmd| {
      cmd.element_count = element_count;
      Some(())
    });
  }

  fn add_image(&mut self, texture: Image, rect: RectangleF32, color: RGBAColor) {
    self.push_image(texture.handle);
    if texture.is_subimage() {
      //
      // add the region inside of the texture
      let uv = [
        Vec2F32::new(
          texture.region[0] as f32 / texture.w as f32,
          texture.region[1] as f32 / texture.h as f32,
        ),
        Vec2F32::new(
          (texture.region[0] + texture.region[2]) as f32 / texture.w as f32,
          (texture.region[1] + texture.region[3]) as f32 / texture.h as f32,
        ),
      ];

      self.push_rect_uv(
        Vec2F32::new(rect.x, rect.y),
        Vec2F32::new(rect.x + rect.w, rect.y + rect.h),
        uv[0],
        uv[1],
        color,
      );
    } else {
      self.push_rect_uv(
        Vec2F32::new(rect.x, rect.y),
        Vec2F32::new(rect.x + rect.w, rect.y + rect.h),
        Vec2F32::same(0_f32),
        Vec2F32::same(1_f32),
        color,
      );
    }
  }

  fn add_text(
    &mut self,
    _font: UserFont,
    _rect: RectangleF32,
    _text: &str,
    _font_height: f32,
    _fg: RGBAColorF32,
  ) {
  }

  pub fn convert(&mut self, cmds: &[Command]) {
    cmds.iter().for_each(|input_cmd| match *input_cmd {
      Command::Scissor(ref s) => {
        self.add_clip(RectangleF32::new(
          s.x as f32,
          s.y as f32,
          s.x as f32 + s.w as f32,
          s.y as f32 + s.h as f32,
        ));
      }

      Command::Line(ref l) => {
        self.stroke_line(
          Vec2F32::new(l.begin.x as f32, l.begin.y as f32),
          Vec2F32::new(l.end.x as f32, l.end.y as f32),
          l.color,
          l.line_thickness as f32,
        );
      }

      Command::Curve(ref c) => {
        self.stroke_curve(
          Vec2F32::new(c.begin.x as f32, c.begin.y as f32),
          Vec2F32::new(c.ctrl[0].x as f32, c.ctrl[0].y as f32),
          Vec2F32::new(c.ctrl[1].x as f32, c.ctrl[1].y as f32),
          Vec2F32::new(c.end.x as f32, c.end.y as f32),
          c.color,
          self.config.curve_segment_count,
          c.line_thickness as f32,
        );
      }

      Command::Rect(ref r) => {
        self.stroke_rect(
          RectangleF32::new(r.x as f32, r.y as f32, r.w as f32, r.h as f32),
          r.color,
          r.rounding as f32,
          r.line_thickness as f32,
        );
      }

      Command::RectFilled(ref r) => {
        self.fill_rect(
          RectangleF32::new(r.x as f32, r.y as f32, r.w as f32, r.h as f32),
          r.color,
          r.rounding as f32,
        );
      }

      Command::RectMulticolor(ref r) => {
        self.fill_rect_multi_color(
          RectangleF32::new(r.x as f32, r.y as f32, r.w as f32, r.h as f32),
          r.left,
          r.top,
          r.right,
          r.bottom,
        );
      }

      Command::Circle(ref c) => {
        self.stroke_circle(
          Vec2F32::new(c.x as f32 + (c.w / 2) as f32, c.y as f32 + (c.h / 2) as f32),
          (c.w / 2) as f32,
          c.color,
          self.config.circle_segment_count,
          c.line_thickness as f32,
        );
      }

      Command::CircleFilled(ref c) => {
        self.fill_circle(
          Vec2F32::new(c.x as f32 + (c.w / 2) as f32, c.y as f32 + (c.h / 2) as f32),
          (c.w / 2) as f32,
          c.color,
          self.config.circle_segment_count,
        );
      }

      Command::Arc(ref a) => {
        self.path_line_to(Vec2F32::new(a.cx as f32, a.cy as f32));
        self.path_arc_to(
          Vec2F32::new(a.cx as f32, a.cy as f32),
          a.r as f32,
          a.a[0],
          a.a[1],
          self.config.arc_segment_count,
        );
        self.path_stroke(a.color, DrawListStroke::Closed, a.line_thickness as f32);
      }

      Command::ArcFilled(ref a) => {
        self.path_line_to(Vec2F32::new(a.cx as f32, a.cy as f32));
        self.path_arc_to(
          Vec2F32::new(a.cx as f32, a.cy as f32),
          a.r as f32,
          a.a[0],
          a.a[1],
          self.config.arc_segment_count,
        );
        self.path_fill(a.color);
      }

      Command::Triangle(ref t) => {
        self.stroke_triangle(
          Vec2F32::new(t.a.x as f32, t.a.y as f32),
          Vec2F32::new(t.b.x as f32, t.b.y as f32),
          Vec2F32::new(t.c.x as f32, t.c.y as f32),
          t.color,
          t.line_thickness as f32,
        );
      }

      Command::TriangleFilled(ref t) => {
        self.fill_triangle(
          Vec2F32::new(t.a.x as f32, t.a.y as f32),
          Vec2F32::new(t.b.x as f32, t.b.y as f32),
          Vec2F32::new(t.c.x as f32, t.c.y as f32),
          t.color,
        );
      }

      Command::Polygon(ref p) => {
        p.points.iter().for_each(|p| {
          let pnt = Vec2F32::new(p.x as f32, p.y as f32);
          self.path_line_to(pnt);
        });
        self.path_stroke(p.color, DrawListStroke::Closed, p.line_thickness as f32);
      }

      Command::PolygonFilled(ref p) => {
        p.points.iter().for_each(|p| {
          let pnt = Vec2F32::new(p.x as f32, p.y as f32);
          self.path_line_to(pnt);
        });

        self.path_fill(p.color);
      }

      Command::Polyline(ref p) => {
        p.points.iter().for_each(|p| {
          let pnt = Vec2F32::new(p.x as f32, p.y as f32);
          self.path_line_to(pnt);
        });
        self.path_stroke(p.color, DrawListStroke::Open, p.line_thickness as f32);
      }

      Command::Image(ref i) => {
        self.add_image(
          i.img,
          RectangleF32::new(i.x as f32, i.y as f32, i.w as f32, i.h as f32),
          i.color,
        );
      }

      _ => {
        println!("Unhandled command");
      }
    });
  }
}