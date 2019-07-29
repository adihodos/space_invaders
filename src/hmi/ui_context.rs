use crate::{
  hmi::{
    base::{
      AntialiasingType, ButtonBehaviour, ConvertConfig, GenericHandle, HashType,
    },
    commands::CommandBuffer,
    input::{Input, MouseButtonId},
    panel::{Panel, PanelFlags, PanelType},
    style::{ConfigurationStacks, Style, StyleItem},
    text_engine::Font,
    vertex_output::{DrawCommand, DrawIndexType, DrawList},
    window::Window,
  },
  math::{colors::RGBAColor, rectangle::RectangleF32, vertex_types::VertexPTC},
};

use enumflags2::BitFlags;
use murmurhash64::murmur_hash64a;
use num::ToPrimitive;
use std::{cell::RefCell, rc::Rc};

pub struct Consts {}

impl Consts {
  pub const VALUE_PAGE_CAPACITY: usize = 48;
}

#[derive(Copy, Clone)]
pub struct Table {
  pub seq:    u32,
  pub size:   u32,
  pub keys:   [u32; Consts::VALUE_PAGE_CAPACITY],
  pub values: [u32; Consts::VALUE_PAGE_CAPACITY],
}

enum PageData {
  Tbl(Table),
  Pan(Panel),
  Win(Window),
}

#[derive(Copy, Clone, Debug)]
enum WindowInsertLocation {
  Front,
  Back,
}

type WindowPtr = Rc<RefCell<Window>>;

pub struct UiContext<'a> {
  pub input:             RefCell<Input>,
  pub style:             Style,
  pub last_widget_state: u32,
  pub button_behviour:   ButtonBehaviour,
  pub stacks:            ConfigurationStacks,
  pub delta_time_sec:    f32,
  draw_list:             DrawList<'a>,
  // TODO: text edit support
  overlay: RefCell<CommandBuffer>,
  // windows
  build:          i32,
  windows:        RefCell<Vec<WindowPtr>>,
  active_win:     RefCell<Option<WindowPtr>>,
  current_win:    RefCell<Option<WindowPtr>>,
  seq:            u32,
  win_handle_seq: usize,
}

impl<'a> UiContext<'a> {
  pub fn new(
    font: Font,
    config: ConvertConfig,
    line_aa: AntialiasingType,
    shape_aa: AntialiasingType,
  ) -> UiContext<'a> {
    Self {
      input:             RefCell::new(Input::new()),
      style:             Style::new(font),
      last_widget_state: 0,
      button_behviour:   ButtonBehaviour::default(),
      stacks:            ConfigurationStacks::default(),
      delta_time_sec:    0f32,
      draw_list:         DrawList::new(config, line_aa, shape_aa),
      overlay:           RefCell::new(CommandBuffer::new(
        Some(RectangleF32::new(
          -8192_f32, -8192_f32, 16834_f32, 16834_f32,
        )),
        128,
      )),
      build:             0,
      windows:           RefCell::new(vec![]),
      current_win:       RefCell::new(None),
      active_win:        RefCell::new(None),
      seq:               0,
      win_handle_seq:    0,
    }
  }

  fn alloc_win_handle(&mut self) -> usize {
    let handle = self.win_handle_seq;
    self.win_handle_seq += 1;
    handle
  }

  pub fn panel_begin(&mut self, title: &str, panel_type: PanelType) -> bool {
    debug_assert!(self.current_win.borrow().is_some());

    if self.current_win.borrow().is_none() {
      return false;
    }

    let winptr = self
      .current_win
      .borrow()
      .as_ref()
      .and_then(|winptr| Some(winptr.clone()))
      .expect("Invalid current window!");

    // reset panel to default state
    winptr.borrow_mut().layout = Box::new(RefCell::new(Panel::new(panel_type)));
    let win_flags = winptr.borrow().flags;

    if win_flags.contains(PanelFlags::WindowHidden | PanelFlags::WindowClosed) {
      return false;
    }

    let scrollbar_size = self.style.window.scrollbar_size;
    let panel_padding = self.style.get_panel_padding(panel_type);

    // window movement
    if win_flags.contains(PanelFlags::WindowMovable)
      && !win_flags.contains(PanelFlags::WindowRom)
    {
      let mut header = winptr.borrow().bounds;
      if Panel::has_header(win_flags, Some(title)) {
        header.h =
          self.style.font.scale + 2f32 * self.style.window.header.padding.y;
        header.h += 2f32 * self.style.window.header.label_padding.y;
      } else {
        header.h = panel_padding.y
      };

      let left_mouse_down = self
        .input
        .borrow()
        .has_mouse_down(MouseButtonId::ButtonLeft);
      let left_mouse_clicked = self
        .input
        .borrow()
        .is_button_clicked(MouseButtonId::ButtonLeft);
      let left_mouse_click_in_cursor = self
        .input
        .borrow()
        .has_mouse_click_down_in_rect(MouseButtonId::ButtonLeft, &header, true);

      if left_mouse_down && left_mouse_click_in_cursor && !left_mouse_clicked {
        let mut w = winptr.borrow_mut();
        w.bounds.x += self.input.borrow().mouse.delta.x;
        w.bounds.y += self.input.borrow().mouse.delta.y;

        let mut input = self.input.borrow_mut();
        let mouse_delta = input.mouse.delta;
        input.mouse.buttons[MouseButtonId::ButtonLeft as usize].clicked_pos +=
          mouse_delta;

        // ctx->style.cursor_active = ctx->style.cursors[NK_CURSOR_MOVE];
      }
    }

    // setup panel
    {
      let win = winptr.borrow();
      let mut layout = win.layout.borrow_mut();
      layout.flags = win_flags;
      layout.bounds = winptr.borrow().bounds;
      layout.bounds.x += panel_padding.x;
      layout.bounds.w -= 2f32 * panel_padding.x;
      if win_flags.contains(PanelFlags::WindowBorder) {
        layout.border = self.style.get_panel_border(panel_type, win_flags);
        layout.bounds = RectangleF32::shrink(&layout.bounds, layout.border);
      } else {
        layout.border = 0f32;
      }

      layout.at_x = layout.bounds.x;
      layout.at_y = layout.bounds.y;
      layout.max_x = 0f32;
      layout.header_height = 0f32;
      layout.footer_height = 0f32;
      // TODO : reset min row
      layout.row.index = 0;
      layout.row.columns = 0;
      layout.row.ratio = std::ptr::null_mut();
      layout.row.item_width = 0f32;
      layout.row.tree_depth = 0;
      layout.row.height = panel_padding.y;
      layout.has_scrolling = true;

      if !win_flags.contains(PanelFlags::WindowNoScrollbar) {
        layout.bounds.w -= scrollbar_size.x;
      }
    }

    // panel header
    if Panel::has_header(win_flags, Some(title)) {
      // calculate header bounds
      let win = winptr.borrow();
      let mut layout = win.layout.borrow_mut();
      let mut header = win.bounds;
      header.h =
        self.style.font.scale + 2f32 * self.style.window.header.padding.y;
      header.h += 2f32 * self.style.window.header.label_padding.y;

      // shrink panel by header
      layout.header_height = header.h;
      layout.bounds.y += header.h;
      layout.bounds.h -= header.h;
      layout.at_y += header.h;

      // select correct header background and text color
      let is_active_win = self
        .active_win
        .borrow()
        .as_ref()
        .map_or(false, |active_win| active_win.borrow().handle == win.handle);

      let (bk, txt_color) = if is_active_win {
        (
          self.style.window.header.active,
          self.style.window.header.label_active,
        )
      } else if self.input.borrow().is_mouse_hovering_rect(&header) {
        (
          self.style.window.header.hover,
          self.style.window.header.label_hover,
        )
      } else {
        (
          self.style.window.header.normal,
          self.style.window.header.label_normal,
        )
      };

      // draw header background
      header.h += 1.0;
      let txt_bk = match bk {
        StyleItem::Img(ref img) => {
          // draw image
          win.buffer.borrow_mut().draw_image(
            header,
            *img,
            RGBAColor::new(255, 255, 255),
          );
          RGBAColor::new_with_alpha(0, 0, 0, 0)
        }

        StyleItem::Color(clr) => {
          // fill rect
          win.buffer.borrow_mut().fill_rect(header, 0f32, clr);
          clr
        }
      };

      // window close button
      {
        // window minimize button
      }

      {
        // window header title
      }
    }

    // draw window background
    let layout_flags = winptr.borrow().layout.borrow().flags;
    if !layout_flags
      .intersects(PanelFlags::WindowMinimized | PanelFlags::WindowDynamic)
    {
      let win = winptr.borrow();
      let layout = win.layout.borrow();
      let body = RectangleF32::new(
        win.bounds.x,
        win.bounds.y + layout.header_height,
        win.bounds.w,
        win.bounds.h - layout.header_height,
      );

      match self.style.window.fixed_background {
        StyleItem::Img(ref img) => win.buffer.borrow_mut().draw_image(
          body,
          *img,
          RGBAColor::new(255, 255, 255),
        ),
        StyleItem::Color(clr) => {
          win.buffer.borrow_mut().fill_rect(body, 0f32, clr)
        }
      }
    }

    // set clipping rectangle
    {
      let buffer_clip = winptr.borrow().buffer.borrow().clip();
      let layout_clip = winptr.borrow().layout.borrow().bounds;
      let clip = RectangleF32::union(&buffer_clip, &layout_clip);
      winptr.borrow().buffer.borrow_mut().push_scissor(clip);
      winptr.borrow().layout.borrow_mut().clip = clip;
    }

    !layout_flags
      .contains(PanelFlags::WindowHidden | PanelFlags::WindowMinimized)
  }
}
