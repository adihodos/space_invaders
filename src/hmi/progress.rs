use crate::{
  hmi::{
    commands::CommandBuffer,
    input::{Input, MouseButtonId},
    style::{StyleItem, StyleProgress},
  },
  math::{colors::RGBAColor, rectangle::RectangleF32, utility::clamp},
};
use enumflags2::BitFlags;
use enumflags2_derive::EnumFlags;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum WidgetLayoutStates {
  Invalid,
  Valid,
  Rom,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, EnumFlags)]
pub enum WidgetStates {
  Modified = 1 << 1,
  Inactive = 1 << 2,
  Entered = 1 << 3,
  Hover = 1 << 4,
  Activated = 1 << 5,
  Left = 1 << 6,
}

impl WidgetStates {
  pub fn is_hovered(s: BitFlags<WidgetStates>) -> bool {
    s.contains(WidgetStates::Hover | WidgetStates::Modified)
  }

  pub fn hovered() -> BitFlags<WidgetStates> {
    WidgetStates::Hover | WidgetStates::Modified
  }

  pub fn is_active(s: BitFlags<WidgetStates>) -> bool {
    s.contains(WidgetStates::Activated | WidgetStates::Modified)
  }

  pub fn active() -> BitFlags<WidgetStates> {
    WidgetStates::Activated | WidgetStates::Modified
  }

  pub fn reset(s: BitFlags<WidgetStates>) -> BitFlags<WidgetStates> {
    if s.contains(WidgetStates::Modified) {
      WidgetStates::Inactive | WidgetStates::Modified
    } else {
      WidgetStates::Inactive | WidgetStates::Inactive
    }
  }
}

pub fn progress_behaviour(
  state: BitFlags<WidgetStates>,
  input: Option<&mut Input>,
  r: &RectangleF32,
  cursor: &RectangleF32,
  max: u32,
  value: u32,
  modifiable: bool,
) -> (BitFlags<WidgetStates>, u32) {
  let mut state = WidgetStates::reset(state);
  if !modifiable {
    return (state, value);
  }

  input.map_or((state, value), |inp| {
    let left_mouse_down = inp.has_mouse_down(MouseButtonId::ButtonLeft);
    let left_mouse_click_in_cursor = inp.has_mouse_click_down_in_rect(
      MouseButtonId::ButtonLeft,
      &cursor,
      true,
    );

    if inp.is_mouse_hovering_rect(&r) {
      state = WidgetStates::hovered();
    }

    let value = if left_mouse_down && left_mouse_click_in_cursor {
      let ratio = 0f32.max((inp.mouse.pos.x - cursor.x) / cursor.w);
      inp.mouse.buttons[MouseButtonId::ButtonLeft as usize]
        .clicked_pos
        .x = cursor.x + cursor.w / 2f32;
      state.insert(WidgetStates::active());
      clamp(0f32, max as f32 * ratio, max as f32) as u32
    } else {
      value
    };

    // set progressbar widget state
    if state.contains(WidgetStates::Hover)
      && !inp.is_mouse_prev_hovering_rect(&r)
    {
      state.insert(WidgetStates::Entered);
    } else if inp.is_mouse_prev_hovering_rect(&r) {
      state.insert(WidgetStates::Left);
    }

    (state, value)
  })
}

pub fn draw_progress(
  cmdbuff: &mut CommandBuffer,
  state: BitFlags<WidgetStates>,
  style: &StyleProgress,
  bounds: &RectangleF32,
  scursor: &RectangleF32,
  _value: u32,
  _max: u32,
) {
  // select correct color/images to draw
  let (bk, cursor) = if state.contains(WidgetStates::Activated) {
    (&style.active, &style.cursor_active)
  } else if state.contains(WidgetStates::Hover) {
    (&style.hover, &style.cursor_hover)
  } else {
    (&style.normal, &style.cursor_normal)
  };

  // draw background
  match bk {
    StyleItem::Img(ref img) => {
      cmdbuff.draw_image(*bounds, *img, RGBAColor::new(255, 255, 255));
    }

    StyleItem::Color(clr) => {
      cmdbuff.fill_rect(*bounds, style.rounding, *clr);
      cmdbuff.stroke_rect(
        *bounds,
        style.rounding,
        style.border,
        style.border_color,
      );
    }
  }

  // draw cursor
  match cursor {
    StyleItem::Img(ref img) => {
      cmdbuff.draw_image(*scursor, *img, RGBAColor::new(255, 255, 255))
    }
    StyleItem::Color(clr) => {
      cmdbuff.fill_rect(*scursor, style.rounding, *clr);
      cmdbuff.stroke_rect(
        *scursor,
        style.rounding,
        style.border,
        style.border_color,
      );
    }
  }
}
