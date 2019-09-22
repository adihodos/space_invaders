use crate::{
  hmi::{
    base::{
      ButtonBehaviour, Heading, TextAlign, WidgetLayoutStates, WidgetStates,
    },
    commands::CommandBuffer,
    input::{Input, MouseButtonId},
    style::{Style, StyleItem, SymbolType},
    text_engine::Font,
    ui_context::WindowPtr,
  },
  math::{
    colors::RGBAColor, rectangle::RectangleF32, utility::clamp, vec2::Vec2F32,
  },
};

use enumflags2::BitFlags;

#[derive(Copy, Clone, Debug)]
pub struct Text {
  pub padding:    Vec2F32,
  pub background: RGBAColor,
  pub text:       RGBAColor,
}

pub fn widget_text(
  out: &mut CommandBuffer,
  b: RectangleF32,
  s: &str,
  t: &Text,
  align: BitFlags<TextAlign>,
  f: Font,
) {
  let b = RectangleF32 {
    h: b.h.max(2f32 * t.padding.y),
    ..b
  };

  let label = RectangleF32 {
    x: 0f32,
    w: 0f32,
    y: b.y + t.padding.y,
    h: f.scale.min(b.h - 2f32 * t.padding.y),
  };

  let text_width = f.text_width(s) + 2f32 * t.padding.x;

  // align in x-axis
  let label = if align.intersects(TextAlign::AlignLeft) {
    RectangleF32 {
      x: b.x + t.padding.x,
      w: 0f32.max(b.w - 2f32 * t.padding.x),
      ..label
    }
  } else if align.intersects(TextAlign::AlignCentered) {
    let w = 1f32.max(2f32 * t.padding.x + text_width);
    let x = b.x + t.padding.x + ((b.w - 2f32 * t.padding.x) - label.w) / 2f32;
    let x = x.max(b.x + t.padding.x);
    let w = (x + w).min(b.x + b.w);
    let w = if w >= x { w - x } else { w };
    RectangleF32 { x, w, ..label }
  } else if align.intersects(TextAlign::AlignRight) {
    let x =
      (b.x + t.padding.x).max(b.x + b.w - 2f32 * t.padding.x + text_width);
    let w = text_width + 2f32 * t.padding.x;
    RectangleF32 { x, w, ..label }
  } else {
    return;
  };

  // align in y-axis
  let label = if align.intersects(TextAlign::AlignMiddle) {
    let y = b.y + b.h * 0.5f32 - f.scale * 0.5f32;
    let h = (b.h * 0.5f32).max(b.h - (b.h * 0.5f32 + f.scale * 0.5f32));
    RectangleF32 { y, h, ..label }
  } else if align.intersects(TextAlign::AlignBottom) {
    RectangleF32 {
      y: b.y + b.h - f.scale,
      h: f.scale,
      ..label
    }
  } else {
    label
  };

  out.draw_text(label, s, f, t.background, t.text);
}

pub fn widget_text_wrap(
  out: &mut CommandBuffer,
  b: RectangleF32,
  s: &str,
  t: &Text,
  f: Font,
) {
  let text = Text {
    padding: Vec2F32::same(0f32),
    ..*t
  };

  let b = RectangleF32 {
    w: b.w.max(2f32 * t.padding.x),
    h: b.h.max(2f32 * t.padding.y) - 2f32 * t.padding.y,
    ..b
  };

  let mut line = RectangleF32 {
    x: b.x + t.padding.x,
    y: b.y + t.padding.y,
    w: b.w - 2f32 * t.padding.x,
    h: 2f32 * t.padding.y + f.scale,
  };

  let (mut fitting, _width) = f.clamp_text(s, line.w);
  let mut done = 0usize;
  while done < s.len() {
    if (fitting <= 0) || (line.y + line.h) >= (b.y + b.h) {
      break;
    }

    widget_text(
      out,
      line,
      &s[done ..],
      &text,
      TextAlign::AlignLeft.into(),
      f,
    );

    done += fitting as usize;
    line.y += f.scale + 2f32 * t.padding.y;
    let (fres, _) = f.clamp_text(&s[done ..], line.w);
    fitting = fres;
  }
}

pub fn text_colored(
  win: WindowPtr,
  style: &Style,
  bounds: RectangleF32,
  s: &str,
  align: BitFlags<TextAlign>,
  color: RGBAColor,
) {
  // let item_padding = style.text.padding;
  let text = Text {
    padding:    style.text.padding,
    background: style.window.background,
    text:       color,
  };

  widget_text(
    &mut win.borrow().buffer.borrow_mut(),
    bounds,
    s,
    &text,
    align,
    style.font,
  );
}

pub fn text_wrap_colored(
  win: WindowPtr,
  style: &Style,
  bounds: RectangleF32,
  s: &str,
  color: RGBAColor,
) {
  // let item_padding = style.text.padding;
  let text = Text {
    padding:    style.text.padding,
    background: style.window.background,
    text:       color,
  };

  widget_text_wrap(
    &mut win.borrow().buffer.borrow_mut(),
    bounds,
    s,
    &text,
    style.font,
  );
}
