use crate::{
  hmi::{
    base::{ButtonBehaviour, Orientation, WidgetStates},
    button::{button_behaviour, do_button_symbol},
    commands::CommandBuffer,
    image::Image,
    input::{Input, KeyId, MouseButtonId},
    style::{StyleItem, StyleScrollbar, SymbolType},
    text_engine::Font,
  },
  math::{
    colors::RGBAColor, rectangle::RectangleF32, utility::clamp, vec2::Vec2F32,
  },
};
use enumflags2::BitFlags;

fn scrollbar_behavior(
  // inp: Option<&mut Input>,
  inp: Option<&std::cell::RefCell<Input>>,
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
  if inp.is_none() {
    return (BitFlags::default(), 0f32);
  }

  let input = inp.unwrap();

  let left_mouse_down =
    input.borrow().has_mouse_down(MouseButtonId::ButtonLeft);
  let left_mouse_clicked =
    input.borrow().has_mouse_click(MouseButtonId::ButtonLeft);
  let left_mouse_click_in_cursor = input.borrow().has_mouse_click_down_in_rect(
    MouseButtonId::ButtonLeft,
    cursor,
    true,
  );

  let mut state = BitFlags::<WidgetStates>::default();
  if input.borrow().is_mouse_hovering_rect(scroll) {
    state.insert(WidgetStates::Hover);
  }
  let scroll_delta = if o == Orientation::Vertical {
    input.borrow().mouse.scroll_delta.y
  } else {
    input.borrow().mouse.scroll_delta.x
  };

  let scroll_offset = if left_mouse_down
    && left_mouse_click_in_cursor
    && !left_mouse_clicked
  {
    // update cursor by mouse dragging
    state = WidgetStates::active();
    if o == Orientation::Vertical {
      let pixel = input.borrow().mouse.delta.y;
      let delta = (pixel / scroll.h) * target;
      let scroll_offset = clamp(0f32, scroll_offset + delta, target - scroll.h);
      let cursor_y = scroll.y + ((scroll_offset / target) * scroll.h);
      input.borrow_mut().mouse.buttons[MouseButtonId::ButtonLeft as usize]
        .clicked_pos
        .y = cursor_y + cursor.h * 0.5f32;
      scroll_offset
    } else {
      let pixel = input.borrow().mouse.delta.x;
      let delta = (pixel / scroll.w) * target;
      let scroll_offset = clamp(0f32, scroll_offset + delta, target - scroll.w);
      let cursor_x = scroll.x + ((scroll_offset / target) * scroll.w);
      input.borrow_mut().mouse.buttons[MouseButtonId::ButtonLeft as usize]
        .clicked_pos
        .x = cursor_x + cursor.w * 0.5f32;
      scroll_offset
    }
  } else if (input.borrow().is_key_pressed(KeyId::KeyScrollUp)
    && o == Orientation::Vertical
    && has_scrolling)
    || button_behaviour(
      *empty0,
      Some(&input.borrow()),
      ButtonBehaviour::ButtonDefault,
    )
    .0
  {
    // scroll page up click on empty space or shortcut
    if o == Orientation::Vertical {
      0f32.max(scroll_offset - scroll.h)
    } else {
      0f32.max(scroll_offset - scroll.w)
    }
  } else if (input.borrow().is_key_pressed(KeyId::KeyScrollDown)
    && o == Orientation::Vertical
    && has_scrolling)
    || button_behaviour(
      *empty1,
      Some(&input.borrow()),
      ButtonBehaviour::ButtonDefault,
    )
    .0
  {
    // scroll page down by click on empty space or shortcut
    if o == Orientation::Vertical {
      (scroll_offset + scroll.h).min(target - scroll.h)
    } else {
      (scroll_offset + scroll.w).min(target - scroll.w)
    }
  } else if has_scrolling {
    if scroll_delta < 0f32 || scroll_delta > 0f32 {
      // update cursor by mouse scrolling
      let scroll_offset = scroll_offset + scroll_step * (-scroll_delta);
      if o == Orientation::Vertical {
        clamp(0f32, scroll_offset, target - scroll.h)
      } else {
        clamp(0f32, scroll_offset, target - scroll.w)
      }
    } else if input.borrow().is_key_pressed(KeyId::KeyScrollStart) {
      // update cursor to the beginning
      if o == Orientation::Vertical {
        0f32
      } else {
        scroll_offset
      }
    } else if input.borrow().is_key_pressed(KeyId::KeyScrollEnd) {
      // update cursor to end
      if o == Orientation::Vertical {
        target - scroll.h
      } else {
        scroll_offset
      }
    } else {
      scroll_offset
    }
  } else {
    scroll_offset
  };

  if state.intersects(WidgetStates::Hover)
    && !input.borrow().is_mouse_prev_hovering_rect(scroll)
  {
    state.insert(WidgetStates::Entered);
  } else if input.borrow().is_mouse_prev_hovering_rect(scroll) {
    state.insert(WidgetStates::Left);
  }

  (state, scroll_offset)
}

pub fn draw_scrollbar(
  out: &mut CommandBuffer,
  state: BitFlags<WidgetStates>,
  style: &StyleScrollbar,
  bounds: &RectangleF32,
  scroll: &RectangleF32,
) {
  // select correct colors/images to draw
  let (background, cursor) = if state.intersects(WidgetStates::Activated) {
    (&style.active, &style.cursor_active)
  } else if state.intersects(WidgetStates::Hover) {
    (&style.hover, &style.cursor_hover)
  } else {
    (&style.normal, &style.cursor_normal)
  };

  // draw background
  match background {
    StyleItem::Color(c) => {
      out.fill_rect(*bounds, style.rounding, *c);
      out.stroke_rect(
        *bounds,
        style.rounding,
        style.border,
        style.border_color,
      );
    }
    StyleItem::Img(i) => {
      out.draw_image(*bounds, *i, RGBAColor::new(255, 255, 255));
    }
  }

  // draw cursor
  match *cursor {
    StyleItem::Color(c) => {
      out.fill_rect(*scroll, style.rounding_cursor, c);
      out.stroke_rect(
        *scroll,
        style.rounding_cursor,
        style.border_cursor,
        style.cursor_border_color,
      );
    }

    StyleItem::Img(i) => {
      out.draw_image(*scroll, i, RGBAColor::new(255, 255, 255));
    }
  }
}

pub fn do_scrollbarv(
  out: &mut CommandBuffer,
  scroll: RectangleF32,
  has_scrolling: bool,
  offset: f32,
  target: f32,
  step: f32,
  button_pixel_inc: f32,
  style: &StyleScrollbar,
  input: Option<&std::cell::RefCell<Input>>,
  font: &Font,
) -> (BitFlags<WidgetStates>, f32) {
  let mut scroll = RectangleF32 {
    w: 1f32.max(scroll.w),
    h: 0f32.max(scroll.h),
    ..scroll
  };

  if target <= scroll.h {
    return (BitFlags::default(), 0f32);
  }

  let mut offset = offset;

  // optional scrollbar buttons
  if style.show_buttons {
    let button = RectangleF32 {
      x: scroll.x,
      y: scroll.y,
      w: scroll.w,
      h: scroll.w,
    };

    let scroll_h = 0f32.max(scroll.h - 2f32 * button.h);
    let scroll_step = step.min(button_pixel_inc);

    // decrement button
    input
      .as_ref()
      .map_or_else(
        || {
          if do_button_symbol(
            &mut BitFlags::default(),
            out,
            button,
            style.dec_symbol,
            ButtonBehaviour::ButtonRepeater,
            &style.dec_button,
            None,
            *font,
          ) {
            Some(())
          } else {
            None
          }
        },
        |cell_input| {
          let inp = cell_input.borrow_mut();
          if do_button_symbol(
            &mut BitFlags::default(),
            out,
            button,
            style.dec_symbol,
            ButtonBehaviour::ButtonRepeater,
            &style.dec_button,
            Some(&mut inp),
            *font,
          ) {
            Some(())
          } else {
            None
          }
        },
      )
      .map(|_| offset -= scroll_step);

    // increment button
    let button = RectangleF32 {
      y: scroll.y + scroll.h - button.h,
      ..button
    };

    input
      .as_ref()
      .map_or_else(
        || {
          if do_button_symbol(
            &mut BitFlags::default(),
            out,
            button,
            style.inc_symbol,
            ButtonBehaviour::ButtonRepeater,
            &style.inc_button,
            None,
            *font,
          ) {
            Some(())
          } else {
            None
          }
        },
        |cell_input| {
          let inp = cell_input.borrow_mut();
          if do_button_symbol(
            &mut BitFlags::default(),
            out,
            button,
            style.inc_symbol,
            ButtonBehaviour::ButtonRepeater,
            &style.inc_button,
            Some(&mut inp),
            *font,
          ) {
            Some(())
          } else {
            None
          }
        },
      )
      .map(|_| offset += scroll_step);

    scroll.y += button.h;
    scroll.h = scroll_h;
  }

  // calculate scrollbar constants
  let scroll_step = step.min(scroll.h);
  let scroll_offset = clamp(0f32, offset, target - scroll.h);
  let scroll_ratio = scroll.h / target;
  let scroll_off = scroll_offset / target;

  // calculate cursor bounds
  let cursor = RectangleF32 {
    h: 0f32.max(
      (scroll_ratio * scroll.h)
        - (2f32 * style.border + 2f32 * style.padding.y),
    ),
    y: scroll.y + (scroll_off * scroll.h) + style.border + style.padding.y,
    w: scroll.w - (2f32 * style.border + 2f32 * style.padding.x),
    x: scroll.x + style.border + style.padding.x,
  };

  // calculate empty space around cursor
  let empty_north = RectangleF32 {
    h: 0f32.max(cursor.y - scroll.y),
    ..scroll
  };

  let empty_south = RectangleF32 {
    y: cursor.y + cursor.h,
    h: 0f32.max((scroll.y + scroll.h) - (cursor.y + cursor.h)),
    ..scroll
  };

  // update scrollbar
  let (state, scroll_offset) = scrollbar_behavior(
    input,
    has_scrolling,
    &scroll,
    &cursor,
    &empty_north,
    &empty_south,
    scroll_offset,
    target,
    scroll_step,
    Orientation::Vertical,
  );

  let scroll_off = scroll_offset / target;
  let cursor = RectangleF32 {
    y: scroll.y
      + (scroll_off * scroll.h)
      + style.border_cursor
      + style.padding.y,
    ..cursor
  };

  // draw scrollbar
  // TODO: custom draw support (draw_begin)
  draw_scrollbar(out, state, style, &scroll, &cursor);
  // TODO: custom draw support (draw_end)

  (state, scroll_offset)
}

pub fn do_scrollbarh(
  out: &mut CommandBuffer,
  scroll: RectangleF32,
  has_scrolling: bool,
  offset: f32,
  target: f32,
  step: f32,
  button_pixel_inc: f32,
  style: &StyleScrollbar,
  inp: Option<&std::cell::RefCell<Input>>,
  font: &Font,
) -> (BitFlags<WidgetStates>, f32) {
  // scrollbar background
  let mut scroll = RectangleF32 {
    h: 1f32.max(scroll.h),
    w: (2f32 * scroll.h).max(scroll.w),
    ..scroll
  };

  if target <= scroll.w {
    return (BitFlags::default(), 0f32);
  }

  let mut offset = offset;

  // optional scrollbar buttons
  if style.show_buttons {
    let button = RectangleF32 {
      x: scroll.x,
      y: scroll.y,
      w: scroll.h,
      h: scroll.h,
    };

    let scroll_w = scroll.w - 2f32 * button.w;
    let scroll_step = step.min(button_pixel_inc);

    // decrement button
    // if do_button_symbol(
    //   &mut BitFlags::default(),
    //   out,
    //   button,
    //   style.dec_symbol,
    //   ButtonBehaviour::ButtonRepeater,
    //   &style.dec_button,
    //   input.as_ref().map_or(None, |i| Some(*i)),
    //   *font,
    // ) {
    //   offset -= scroll_step;
    // }

    // increment button
    let button = RectangleF32 {
      x: scroll.x + scroll.w - button.w,
      ..button
    };
    // if do_button_symbol(
    //   &mut BitFlags::default(),
    //   out,
    //   button,
    //   style.inc_symbol,
    //   ButtonBehaviour::ButtonRepeater,
    //   &style.inc_button,
    //   input.as_ref().map_or(None, |i| Some(*i)),
    //   *font,
    // ) {
    //   offset += scroll_step;
    // }

    scroll.x += button.w;
    scroll.w = scroll_w;
  }

  // calculate scrollbar constants
  let scroll_step = step.min(scroll.w);
  let scroll_offset = clamp(0f32, offset, target - scroll.w);
  let scroll_ratio = scroll.w / target;
  let scroll_off = scroll_offset / target;

  // calculate cursor bounds
  let cursor = RectangleF32 {
    w: scroll_ratio * scroll.w - (2f32 * style.border + 2f32 * style.padding.x),
    x: scroll.x + (scroll_off * scroll.w) + style.border + style.padding.x,
    h: scroll.h - (2f32 * style.border + 2f32 * style.padding.y),
    y: scroll.y + style.border + style.padding.y,
  };

  // calculate empty space around cursor
  let empty_west = RectangleF32 {
    w: cursor.x - scroll.x,
    ..scroll
  };

  let empty_east = RectangleF32 {
    x: cursor.x + cursor.w,
    w: (scroll.x + scroll.w) - (cursor.x + cursor.w),
    ..scroll
  };

  // update scrollbar
  let (state, scroll_offset) = scrollbar_behavior(
    inp,
    has_scrolling,
    &scroll,
    &cursor,
    &empty_west,
    &empty_east,
    scroll_offset,
    target,
    scroll_step,
    Orientation::Horizontal,
  );

  let scroll_off = scroll_offset / target;
  let cursor = RectangleF32 {
    x: scroll.x + (scroll_off * scroll.w),
    ..cursor
  };

  // draw scrollbar
  // TODO: custom draw support (draw_begin)
  draw_scrollbar(out, state, style, &scroll, &cursor);
  // TODO: custom draw support (draw_end)

  (state, scroll_offset)
}
