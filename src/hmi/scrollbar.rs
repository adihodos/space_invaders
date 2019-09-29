use crate::{
  hmi::{
    base::{Orientation, WidgetStates},
    button::button_behaviour,
    commands::CommandBuffer,
    image::Image,
    input::{Input, KeyId, MouseButtonId},
    style::{StyleButton, StyleItem, SymbolType},
  },
  math::{
    colors::RGBAColor, rectangle::RectangleF32, utility::clamp, vec2::Vec2F32,
  },
};
use enumflags2::BitFlags;

fn scrollbar_behavior(
  inp: Option<&mut Input>,
  has_scrolling: bool,
  scroll: &RectangleF32,
  cursor: &RectangleF32,
  empty0: &RectangleF32,
  empty1: &RectangleF32,
  scroll_offset: f32,
  target: f32,
  scroll_step: f32,
  o: Orientation,
) -> (BitFlags<WidgetStates>, f32) {
  inp.map_or((BitFlags::default(), 0f32), |mut inpt| {
    let left_mouse_down = inpt.has_mouse_down(MouseButtonId::ButtonLeft);
    let left_mouse_clicked = inpt.has_mouse_click(MouseButtonId::ButtonLeft);
    let left_mouse_click_in_cursor = inpt.has_mouse_click_down_in_rect(
      MouseButtonId::ButtonLeft,
      cursor,
      true,
    );

    let mut state = BitFlags::<WidgetStates>::default();
    if inpt.is_mouse_hovering_rect(scroll) {
      state.insert(WidgetStates::Hover);
    }
    let scroll_delta = if o == Orientation::Vertical {
      inpt.mouse.scroll_delta.y
    } else {
      inpt.mouse.scroll_delta.x
    };

    let scroll_offset =
      if left_mouse_down && left_mouse_click_in_cursor && !left_mouse_clicked {
        // update cursor by mouse dragging
        state = WidgetStates::active();
        if o == Orientation::Vertical {
          let pixel = inpt.mouse.delta.y;
          let delta = (pixel / scroll.h) * target;
          let scroll_offset =
            clamp(0f32, scroll_offset + delta, target - scroll.h);
          let cursor_y = scroll.y + ((scroll_offset / target) * scroll.h);
          inpt.mouse.buttons[MouseButtonId::ButtonLeft as usize]
            .clicked_pos
            .y = cursor_y + cursor.h * 0.5f32;
          scroll_offset
        } else {
          let pixel = inpt.mouse.delta.x;
          let delta = (pixel / scroll.w) * target;
          let scroll_offset =
            clamp(0f32, scroll_offset + delta, target - scroll.w);
          let cursor_x = scroll.x + ((scroll_offset / target) * scroll.w);
          inpt.mouse.buttons[MouseButtonId::LeftButton as usize]
            .clicked_pos
            .x = cursor_x + cursor.w * 0.5f32;
          scroll_offset
        }
      } else if (inpt.is_key_pressed(KeyId::ScrollUp)
        && o == Orientation::Vertical
        && has_scrolling)
        || button_behavior(empty0, Some(inpt), 
      {
        0f32
      } else {
        0f32
      };

    (state, scroll_offset)
  })
}
