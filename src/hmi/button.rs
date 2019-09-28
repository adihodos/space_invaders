use crate::{
  hmi::{
    base::{
      triangle_from_direction, ButtonBehaviour, Heading, TextAlign,
      WidgetStates,
    },
    commands::CommandBuffer,
    image::Image,
    input::{Input, MouseButtonId},
    style::{StyleButton, StyleItem, SymbolType},
    text::{widget_text, Text},
    text_engine::Font,
  },
  math::{colors::RGBAColor, rectangle::RectangleF32, vec2::Vec2F32},
};
use enumflags2::BitFlags;

fn draw_symbol(
  out: &mut CommandBuffer,
  typ: SymbolType,
  content: RectangleF32,
  background: RGBAColor,
  foreground: RGBAColor,
  border_width: f32,
  font: Font,
) {
  match typ {
    SymbolType::X
    | SymbolType::Underscore
    | SymbolType::Plus
    | SymbolType::Minus => {
      // single character text symbol
      let _txt = match typ {
        SymbolType::X => "x",
        SymbolType::Underscore => "_",
        SymbolType::Plus => "+",
        _ => "-",
      };

      widget_text(
        out,
        content,
        &_txt,
        &Text {
          padding: Vec2F32::same(0f32),
          background,
          text: foreground,
        },
        TextAlign::centered(),
        font,
      );
    }

    SymbolType::CircleSolid
    | SymbolType::CircleOutline
    | SymbolType::RectSolid
    | SymbolType::RectOutline => {
      // simple empty or filled shapes
      if typ == SymbolType::RectSolid || typ == SymbolType::RectOutline {
        out.fill_rect(content, 0f32, foreground);
        if typ == SymbolType::RectOutline {
          out.fill_rect(
            RectangleF32::shrink(&content, border_width),
            0f32,
            background,
          );
        }
      } else {
        out.fill_circle(content, foreground);
        if typ == SymbolType::CircleOutline {
          out.fill_circle(RectangleF32::shrink(&content, 1f32), background);
        }
      }
    }

    SymbolType::TriangleUp
    | SymbolType::TriangleDown
    | SymbolType::TriangleLeft
    | SymbolType::TriangleRight => {
      let heading = match typ {
        SymbolType::TriangleUp => Heading::Up,
        SymbolType::TriangleDown => Heading::Down,
        SymbolType::TriangleLeft => Heading::Left,
        _ => Heading::Right,
      };

      let (a, b, c) = triangle_from_direction(content, 0f32, 0f32, heading);
      out.fill_triangle(a.x, a.y, b.x, b.y, c.x, c.y, foreground);
    }
    _ => {}
  };
}

fn button_behaviour(
  state: &mut BitFlags<WidgetStates>,
  r: RectangleF32,
  i: Option<&Input>,
  behavior: ButtonBehaviour,
) -> bool {
  *state = WidgetStates::reset(*state);
  i.map_or(false, |i| {
    let result = if i.is_mouse_hovering_rect(&r) {
      *state = WidgetStates::Hover.into();

      if i.is_mouse_down(MouseButtonId::ButtonLeft) {
        *state = WidgetStates::active();
      }

      if i.has_mouse_click_in_rect(MouseButtonId::ButtonLeft, &r) {
        if behavior != ButtonBehaviour::ButtonDefault {
          i.is_mouse_down(MouseButtonId::ButtonLeft)
        } else {
          i.is_mouse_pressed(MouseButtonId::ButtonLeft)
        }
      } else {
        false
      }
    } else {
      false
    };

    if state.contains(WidgetStates::Hover) && !i.is_mouse_prev_hovering_rect(&r)
    {
      state.insert(WidgetStates::Entered);
    } else if i.is_mouse_prev_hovering_rect(&r) {
      state.insert(WidgetStates::Left);
    }

    result
  })
}

pub fn draw_button<'a>(
  out: &mut CommandBuffer,
  bounds: &RectangleF32,
  state: BitFlags<WidgetStates>,
  style: &'a StyleButton,
) -> &'a StyleItem {
  let background = if state.contains(WidgetStates::Hover) {
    &style.hover
  } else if state.contains(WidgetStates::Activated) {
    &style.active
  } else {
    &style.normal
  };

  match background {
    StyleItem::Img(ref i) => {
      out.draw_image(*bounds, *i, RGBAColor::new(255, 255, 255));
    }

    StyleItem::Color(ref c) => {
      out.fill_rect(*bounds, style.rounding, *c);
      out.stroke_rect(
        *bounds,
        style.rounding,
        style.border,
        style.border_color,
      );
    }
  }

  background
}

pub fn do_button(
  state: &mut BitFlags<WidgetStates>,
  _out: &mut CommandBuffer,
  r: RectangleF32,
  style: &StyleButton,
  i: Option<&Input>,
  behavior: ButtonBehaviour,
) -> (bool, RectangleF32) {
  let bounds = RectangleF32 {
    x: r.x - style.touch_padding.x,
    y: r.y - style.touch_padding.y,
    w: r.w + 2f32 * style.touch_padding.x,
    h: r.h + 2f32 * style.touch_padding.y,
  };

  let content = RectangleF32 {
    x: r.x + style.padding.x + style.border + style.rounding,
    y: r.y + style.padding.y + style.border + style.rounding,
    w: r.w - (2f32 * style.padding.x + style.border + 2f32 * style.rounding),
    h: r.h - (2f32 * style.padding.y + style.border + 2f32 * style.rounding),
  };

  (button_behaviour(state, bounds, i, behavior), content)
}

fn draw_button_text(
  out: &mut CommandBuffer,
  bounds: &RectangleF32,
  content: &RectangleF32,
  state: BitFlags<WidgetStates>,
  style: &StyleButton,
  txt: &str,
  alignment: BitFlags<TextAlign>,
  font: Font,
) {
  let background = draw_button(out, bounds, state, style);

  // select correct colors/images
  let background = match background {
    StyleItem::Color(c) => *c,
    _ => style.text_background,
  };

  let text = if state.intersects(WidgetStates::Hover) {
    style.text_hover
  } else if state.intersects(WidgetStates::Activated) {
    style.text_active
  } else {
    style.text_normal
  };

  widget_text(
    out,
    *content,
    txt,
    &Text {
      background,
      text,
      padding: Vec2F32::same(0f32),
    },
    alignment,
    font,
  );
}

pub fn do_button_text(
  state: &mut BitFlags<WidgetStates>,
  out: &mut CommandBuffer,
  bounds: RectangleF32,
  s: &str,
  align: BitFlags<TextAlign>,
  behavior: ButtonBehaviour,
  style: &StyleButton,
  i: Option<&Input>,
  font: Font,
) -> bool {
  let (res, content) = do_button(state, out, bounds, style, i, behavior);
  // TODO: draw begin support
  draw_button_text(out, &bounds, &content, *state, style, s, align, font);
  // TODO: draw end support
  res
}

fn draw_button_symbol(
  out: &mut CommandBuffer,
  bounds: &RectangleF32,
  content: &RectangleF32,
  state: BitFlags<WidgetStates>,
  style: &StyleButton,
  typ: SymbolType,
  font: Font,
) {
  // select correct colors/images
  let background = draw_button(out, bounds, state, style);

  let bg = match background {
    StyleItem::Color(c) => *c,
    _ => style.text_background,
  };

  let sym = if state.intersects(WidgetStates::Hover) {
    style.text_hover
  } else if state.intersects(WidgetStates::Activated) {
    style.text_active
  } else {
    style.text_normal
  };

  draw_symbol(out, typ, *content, bg, sym, 1f32, font);
}

pub fn do_button_symbol(
  state: &mut BitFlags<WidgetStates>,
  out: &mut CommandBuffer,
  bounds: RectangleF32,
  symbol: SymbolType,
  behavior: ButtonBehaviour,
  style: &StyleButton,
  i: Option<&Input>,
  font: Font,
) -> bool {
  let (res, content) = do_button(state, out, bounds, style, i, behavior);
  // TODO: support for custom drawing (draw_begin)
  draw_button_symbol(out, &bounds, &content, *state, style, symbol, font);
  // TODO: support for custom drawing (draw_end)

  res
}

fn draw_button_image(
  out: &mut CommandBuffer,
  bounds: &RectangleF32,
  content: &RectangleF32,
  state: BitFlags<WidgetStates>,
  style: &StyleButton,
  img: &Image,
) {
  draw_button(out, bounds, state, style);
  out.draw_image(*content, *img, RGBAColor::new(255, 255, 255));
}

pub fn do_button_image(
  state: &mut BitFlags<WidgetStates>,
  out: &mut CommandBuffer,
  bounds: RectangleF32,
  img: Image,
  behavior: ButtonBehaviour,
  style: &StyleButton,
  i: Option<&Input>,
) -> bool {
  let (res, content) = do_button(state, out, bounds, style, i, behavior);
  let content = RectangleF32 {
    x: content.x + style.image_padding.x,
    y: content.y + style.image_padding.y,
    w: content.w - 2f32 * style.image_padding.x,
    h: content.h - 2f32 * style.image_padding.y,
  };

  // TODO: support for custom drawing (draw_begin)
  draw_button_image(out, &bounds, &content, *state, style, &img);
  // TODO: support for custom drawing (draw_end)

  res
}

fn draw_button_text_symbol(
  out: &mut CommandBuffer,
  bounds: &RectangleF32,
  label: &RectangleF32,
  symbol: &RectangleF32,
  state: BitFlags<WidgetStates>,
  style: &StyleButton,
  s: &str,
  typ: SymbolType,
  font: Font,
) {
  // select correct background colors/images
  let background = draw_button(out, bounds, state, style);

  let sym = if state.intersects(WidgetStates::Hover) {
    style.text_hover
  } else if state.intersects(WidgetStates::Activated) {
    style.text_active
  } else {
    style.text_normal
  };

  draw_symbol(out, typ, *symbol, style.text_background, sym, 0f32, font);

  let text = Text {
    background: match background {
      StyleItem::Color(c) => *c,
      _ => style.text_background,
    },
    text:       if state.intersects(WidgetStates::Hover) {
      style.text_hover
    } else if state.intersects(WidgetStates::Activated) {
      style.text_active
    } else {
      style.text_normal
    },
    padding:    Vec2F32::same(0f32),
  };

  widget_text(out, *label, s, &text, TextAlign::centered(), font);
}

pub fn do_button_text_symbol(
  state: &mut BitFlags<WidgetStates>,
  out: &mut CommandBuffer,
  bounds: RectangleF32,
  symbol: SymbolType,
  s: &str,
  align: BitFlags<TextAlign>,
  behavior: ButtonBehaviour,
  style: &StyleButton,
  f: Font,
  i: Option<&Input>,
) -> bool {
  let (result, content_rect) =
    do_button(state, out, bounds, style, i, behavior);

  let tri = RectangleF32 {
    y: content_rect.y + content_rect.h * 0.5f32 - f.scale * 0.5f32,
    w: f.scale,
    h: f.scale,
    x: if align.intersects(TextAlign::AlignLeft) {
      0f32.max(
        content_rect.x + content_rect.w - (2f32 * style.padding.x + f.scale),
      )
    } else {
      content_rect.x + 2f32 * style.padding.x
    },
  };

  // TODO: custom draw support (draw_begin)
  draw_button_text_symbol(
    out,
    &bounds,
    &content_rect,
    &tri,
    *state,
    style,
    s,
    symbol,
    f,
  );
  // TODO: custom draw support (draw_end)
  result
}

fn draw_button_text_image(
  out: &mut CommandBuffer,
  bounds: &RectangleF32,
  label: &RectangleF32,
  image: &RectangleF32,
  state: BitFlags<WidgetStates>,
  style: &StyleButton,
  s: &str,
  font: Font,
  img: &Image,
) {
  let background = draw_button(out, bounds, state, style);

  // select correct colors
  let text = Text {
    background: match background {
      StyleItem::Color(c) => *c,
      _ => style.text_background,
    },
    text:       if state.intersects(WidgetStates::Hover) {
      style.text_hover
    } else if state.intersects(WidgetStates::Activated) {
      style.text_active
    } else {
      style.text_normal
    },
    padding:    Vec2F32::same(0f32),
  };

  widget_text(out, *label, s, &text, TextAlign::centered(), font);
  out.draw_image(*image, *img, RGBAColor::new(255, 255, 255));
}

pub fn do_button_text_image(
  state: &mut BitFlags<WidgetStates>,
  out: &mut CommandBuffer,
  bounds: RectangleF32,
  img: Image,
  s: &str,
  align: BitFlags<TextAlign>,
  behavior: ButtonBehaviour,
  style: &StyleButton,
  font: Font,
  i: Option<&Input>,
) -> bool {
  let (result, content) = do_button(state, out, bounds, style, i, behavior);
  let icon = RectangleF32 {
    y: bounds.y + style.padding.y,
    h: bounds.h - 2f32 * style.padding.y,
    w: bounds.h - 2f32 * style.padding.y,
    x: if align.intersects(TextAlign::AlignLeft) {
      0f32.max(
        bounds.x + bounds.w - 2f32 * style.padding.x + bounds.h
          - 2f32 * style.padding.y,
      )
    } else {
      bounds.x + 2f32 * style.padding.x
    },
  };

  let icon = RectangleF32 {
    x: icon.x + style.image_padding.x,
    y: icon.y + style.image_padding.y,
    w: icon.w - 2f32 * style.image_padding.x,
    h: icon.h - 2f32 * style.image_padding.y,
  };

  // TODO: custom draw support (draw_begin)
  draw_button_text_image(
    out, &bounds, &content, &icon, *state, style, s, font, &img,
  );
  // TODO: custom draw support (draw_end)

  result
}
