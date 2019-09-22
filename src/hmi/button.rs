use crate::{
  hmi::{
    base::{
      triangle_from_direction, ButtonBehaviour, Heading, WidgetLayoutStates,
      WidgetStates,
    },
    commands::CommandBuffer,
    input::{Input, MouseButtonId},
    style::{StyleButton, StyleItem, SymbolType},
    text_engine::Font,
  },
  math::{
    colors::RGBAColor, rectangle::RectangleF32, utility::clamp, vec2::Vec2F32,
  },
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

      // TODO: fix this by implementing widget_text
      // struct nk_text text;
      // text.padding = nk_vec2(0,0);
      // text.background = background;
      // text.text = foreground;
      // nk_widget_text(out, content, X, 1, &text, NK_TEXT_CENTERED, font);
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
  i: &Input,
  behavior: ButtonBehaviour,
) -> bool {
  *state = WidgetStates::reset(*state);
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

  if state.contains(WidgetStates::Hover) && !i.is_mouse_prev_hovering_rect(&r) {
    state.insert(WidgetStates::Entered);
  } else if i.is_mouse_prev_hovering_rect(&r) {
    state.insert(WidgetStates::Left);
  }

  result
}

fn draw_button<'a>(
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

fn do_button(
  state: &mut BitFlags<WidgetStates>,
  _out: &mut CommandBuffer,
  r: RectangleF32,
  style: &StyleButton,
  i: &Input,
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
