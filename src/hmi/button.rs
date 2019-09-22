use crate::{
  hmi::{
    base::{
      triangle_from_direction, Heading, WidgetLayoutStates, WidgetStates,
    },
    commands::CommandBuffer,
    input::{Input, MouseButtonId},
    style::{StyleItem, SymbolType},
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

fn button_behaviour(state: &mut WidgetLayoutStates) -> bool {}
