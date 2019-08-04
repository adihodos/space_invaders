use crate::{
  hmi::{
    base::{
      AntialiasingType, ButtonBehaviour, Consts, ConvertConfig, GenericHandle,
      HashType, WidgetLayoutStates,
    },
    commands::CommandBuffer,
    input::{Input, MouseButtonId},
    panel::{LayoutFormat, Panel, PanelFlags, PanelRowLayoutType, PanelType},
    style::{ConfigurationStacks, Style, StyleItem},
    text_engine::Font,
    vertex_output::{DrawCommand, DrawIndexType, DrawList},
    window::Window,
  },
  math::{
    colors::RGBAColor,
    rectangle::RectangleF32,
    utility::{clamp, saturate},
    vec2::Vec2F32,
    vertex_types::VertexPTC,
  },
};

use enumflags2::BitFlags;
use murmurhash64::murmur_hash64a;
use num::ToPrimitive;
use std::{cell::RefCell, rc::Rc};

// pub struct Consts {}

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

  pub fn panel_begin(
    &mut self,
    title: &str,
    panel_type: BitFlags<PanelType>,
  ) -> bool {
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

        // TODO: fix this shite
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
      layout.reset_min_row_height(&self.style);
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

      if !layout.is_nonblock() {
        layout.footer_height = 0f32;
        if !win_flags
          .contains(PanelFlags::WindowNoScrollbar | PanelFlags::WindowScalable)
        {
          layout.footer_height = scrollbar_size.y;
        }
        layout.bounds.h -= layout.footer_height;
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
      .intersects(PanelFlags::WindowHidden | PanelFlags::WindowMinimized)
  }

  pub fn panel_end(&self) {
    debug_assert!(self.current_win.borrow().is_some());

    self
      .current_win
      .borrow()
      .as_ref()
      .and_then(|winptr| Some(winptr.clone()))
      .and_then(|win| {
        let win = win.borrow();
        let mut layout = win.layout.borrow_mut();

        if !layout.is_sub() {
          win.buffer.borrow_mut().push_scissor(Consts::null_rect());
        }

        let scrollbar_size = self.style.window.scrollbar_size;
        let panel_padding = self.style.get_panel_padding(layout.typ);

        // update the current cursor Y-position to point over the last added
        // widget
        layout.at_y += layout.row.height;

        // dynamic panels
        if layout.flags.contains(PanelFlags::WindowDynamic)
          && !layout.flags.contains(PanelFlags::WindowMinimized)
        {
          // update panel height to fit dynamic growth
          if layout.at_y < (layout.bounds.y + layout.bounds.h) {
            layout.bounds.h = layout.at_y - layout.bounds.y;
          }

          // fill top empty space
          let empty_space = RectangleF32 {
            h: panel_padding.y,
            ..win.bounds
          };
          win.buffer.borrow_mut().fill_rect(
            empty_space,
            0f32,
            self.style.window.background,
          );

          // fill left empty space
          let empty_space = RectangleF32 {
            x: win.bounds.x,
            y: layout.bounds.y,
            w: panel_padding.x + layout.border,
            h: layout.bounds.h,
          };
          win.buffer.borrow_mut().fill_rect(
            empty_space,
            0f32,
            self.style.window.background,
          );

          // fill right empty space
          let adjust_for_scrollbar = if unsafe { *layout.offset_y } == 0
            && !layout.flags.contains(PanelFlags::WindowNoScrollbar)
          {
            scrollbar_size.x
          } else {
            0f32
          };

          let empty_space = RectangleF32 {
            x: layout.bounds.x + layout.bounds.w,
            y: layout.bounds.y,
            w: panel_padding.x + layout.border + adjust_for_scrollbar,
            h: layout.bounds.h,
          };
          win.buffer.borrow_mut().fill_rect(
            empty_space,
            0f32,
            self.style.window.background,
          );

          // fill bottom empty space
          if layout.footer_height > 0f32 {
            let empty_space = RectangleF32 {
              y: layout.bounds.y + layout.bounds.h,
              h: layout.footer_height,
              ..win.bounds
            };
            win.buffer.borrow_mut().fill_rect(
              empty_space,
              0f32,
              self.style.window.background,
            );
          }
        }

        // TODO: scrollbars
        // TODO: hide scroll if no user input

        // window border
        if layout.flags.contains(PanelFlags::WindowBorder) {
          let padding_y = if layout.flags.contains(PanelFlags::WindowMinimized)
          {
            self.style.window.border + win.bounds.y + layout.header_height
          } else {
            if layout.flags.contains(PanelFlags::WindowDynamic) {
              layout.bounds.y + layout.bounds.h + layout.footer_height
            } else {
              win.bounds.y + win.bounds.h
            }
          };

          let border = RectangleF32 {
            h: padding_y - win.bounds.y,
            ..win.bounds
          };
          win.buffer.borrow_mut().fill_rect(
            border,
            0f32,
            self.style.get_panel_border_color(layout.typ),
          );
        }

        // scaler
        let draw_scaler = layout.flags.contains(PanelFlags::WindowScalable)
          && !layout.flags.intersects(
            PanelFlags::WindowMinimized
              | PanelFlags::WindowRom
              | PanelFlags::WindowNoInput,
          );

        if draw_scaler {
          // calculate scaler bounds
          let x = layout.flags.contains(PanelFlags::WindowNoScrollbar) as i32
            as f32
            * (-scrollbar_size.x)
            + if layout.flags.contains(PanelFlags::WindowScaleLeft) {
              layout.bounds.x - panel_padding.x * 0.5f32
            } else {
              layout.bounds.x + layout.bounds.w + panel_padding.x
            };

          let scaler = RectangleF32 {
            x,
            y: layout.bounds.y + layout.bounds.h,
            w: scrollbar_size.x,
            h: scrollbar_size.y,
          };

          // draw scaler
          match self.style.window.scaler {
            StyleItem::Img(ref img) => {
              win.buffer.borrow_mut().draw_image(
                scaler,
                *img,
                RGBAColor::new(255, 255, 255),
              );
            }

            StyleItem::Color(c) => {
              if layout.flags.contains(PanelFlags::WindowScaleLeft) {
                win.buffer.borrow_mut().fill_triangle(
                  scaler.x,
                  scaler.y,
                  scaler.x,
                  scaler.y + scaler.h,
                  scaler.x + scaler.w,
                  scaler.y + scaler.h,
                  c,
                );
              } else {
                win.buffer.borrow_mut().fill_triangle(
                  scaler.x + scaler.w,
                  scaler.y,
                  scaler.x + scaler.w,
                  scaler.y + scaler.h,
                  scaler.x,
                  scaler.y + scaler.h,
                  c,
                );
              }
            }
          }

          // do window scaling
          if !win.flags.contains(PanelFlags::WindowRom) {
            let left_mouse_down = self
              .input
              .borrow()
              .has_mouse_down(MouseButtonId::ButtonLeft);
            let left_mouse_click_in_scaler =
              self.input.borrow().has_mouse_click_down_in_rect(
                MouseButtonId::ButtonLeft,
                &scaler,
                true,
              );

            if left_mouse_down && left_mouse_click_in_scaler {
              let delta_x =
                if layout.flags.contains(PanelFlags::WindowScaleLeft) {
                  win.bounds.x += self.input.borrow().mouse.delta.x;
                  -self.input.borrow().mouse.delta.x
                } else {
                  self.input.borrow().mouse.delta.x
                };
            }
          }
        }

        Some(())
      });
  }

  /// progress bar
  pub fn progress(
    &mut self,
    cur: u32,
    max: u32,
    modifiable: bool,
  ) -> (bool, u32) {
    debug_assert!(self.current_win.borrow().is_some());

    use crate::hmi::progress::do_progress;
    (false, 0)
  }

  pub fn prog(&mut self, cur: u32, max: u32, modifyable: bool) -> u32 {
    let (_, cur) = self.progress(cur, max, modifyable);
    cur
  }

  fn layout_row_calculate_usable_space(
    style: &Style,
    typ: BitFlags<PanelType>,
    total_space: f32,
    columns: i32,
  ) -> f32 {
    let spacing = style.window.spacing;
    let padding = style.get_panel_padding(typ);
    // calculate usable panel space
    let panel_padding = 2f32 * padding.x;
    let panel_spacing = (columns - 1).max(0) as f32 * spacing.x;
    total_space - panel_padding - panel_spacing
  }

  fn panel_layout(&self, win: &Window, height: f32, cols: i32) {
    //  if one of these triggers you forgot to add an `if` condition around
    // either a window, group, popup, combobox or contextual menu `begin`
    // and `end` block. Example:
    // if (nk_begin(...) {...} nk_end(...); or
    // if (nk_group_begin(...) { nk_group_end(...);}

    let mut layout = win.layout.borrow_mut();
    let style = &self.style;

    debug_assert!(!layout.flags.contains(PanelFlags::WindowMinimized));
    debug_assert!(!layout.flags.contains(PanelFlags::WindowHidden));
    debug_assert!(!layout.flags.contains(PanelFlags::WindowClosed));

    let item_spacing = style.window.spacing;
    layout.row.index = 0;
    layout.at_y += layout.row.height;
    layout.row.columns = cols;
    layout.row.height = if height == 0f32 {
      height.max(layout.row.min_height) + item_spacing.y
    } else {
      height + item_spacing.y
    };

    layout.row.item_offset = 0f32;

    if layout.flags.contains(PanelFlags::WindowDynamic) {
      // draw background for dynamic panels
      let bk = RectangleF32::new(
        win.bounds.x,
        layout.at_y - 1f32,
        win.bounds.w,
        layout.row.height + 1f32,
      );
      win
        .buffer
        .borrow_mut()
        .fill_rect(bk, 0f32, style.window.background);
    }
  }

  pub fn row_layout(
    &self,
    fmt: LayoutFormat,
    height: f32,
    cols: i32,
    width: i32,
  ) {
    debug_assert!(self.current_win.borrow().is_some());

    self
      .current_win
      .borrow()
      .as_ref()
      // .and_then(|winptr| Some(winptr.clone()))
      .and_then(|winptr| {
        self.panel_layout(&winptr.borrow(), height, cols);
        if fmt == LayoutFormat::Dynamic {
          winptr.borrow().layout.borrow_mut().row.typ =
            PanelRowLayoutType::DynamicFixed;
        } else {
          winptr.borrow().layout.borrow_mut().row.typ =
            PanelRowLayoutType::StaticFixed;
        }

        let win = winptr.borrow();
        let mut layout = win.layout.borrow_mut();
        layout.row.ratio = std::ptr::null_mut();
        layout.row.filled = 0f32;
        layout.row.item_offset = 0f32;
        layout.row.item_width = width as f32;
        Some(())
      });
  }

  pub fn layout_ratio_from_pixel(&self, pixel_width: f32) -> f32 {
    self.current_win.borrow().as_ref().map_or(0f32, |winptr| {
      clamp(0f32, pixel_width / winptr.borrow().bounds.x, 1f32)
    })
  }

  pub fn layout_row_dynamic(&self, height: f32, cols: i32) {
    self.row_layout(LayoutFormat::Dynamic, height, cols, 0)
  }

  pub fn layout_row_static(&self, height: f32, item_width: i32, cols: i32) {
    self.row_layout(LayoutFormat::Static, height, cols, item_width)
  }

  pub fn layout_row_begin(
    &self,
    fmt: LayoutFormat,
    row_height: f32,
    cols: i32,
  ) {
    debug_assert!(self.current_win.borrow().is_some());

    self.current_win.borrow().as_ref().and_then(|winptr| {
      let win = winptr.borrow();

      self.panel_layout(&win, row_height, cols);
      let mut layout = win.layout.borrow_mut();
      layout.row.typ = if fmt == LayoutFormat::Dynamic {
        PanelRowLayoutType::DynamicRow
      } else {
        PanelRowLayoutType::StaticRow
      };

      layout.row.ratio = std::ptr::null_mut();
      layout.row.filled = 0f32;
      layout.row.item_width = 0f32;
      layout.row.item_offset = 0f32;
      layout.row.columns = cols;
      Some(())
    });
  }

  pub fn layout_row_push(&self, ratio_or_width: f32) {
    debug_assert!(self.current_win.borrow().is_some());

    self.current_win.borrow().as_ref().and_then(|winptr| {
      let win = winptr.borrow();

      let mut layout = win.layout.borrow_mut();
      if layout.row.typ != PanelRowLayoutType::StaticRow
        || layout.row.typ != PanelRowLayoutType::DynamicRow
      {
        return Some(());
      }

      if layout.row.typ == PanelRowLayoutType::DynamicRow {
        let ratio = ratio_or_width;
        if (ratio + layout.row.filled) > 1f32 {
          return Some(());
        }

        layout.row.item_width = if ratio > 0f32 {
          saturate(ratio)
        } else {
          1f32 - layout.row.filled
        };
      } else {
        layout.row.item_width = ratio_or_width;
      }

      Some(())
    });
  }

  pub fn layout_row_end(&self) {
    debug_assert!(self.current_win.borrow().is_some());

    self.current_win.borrow().as_ref().and_then(|winptr| {
      let win = winptr.borrow();
      let mut layout = win.layout.borrow_mut();
      debug_assert!(layout.row.typ == PanelRowLayoutType::StaticRow);
      debug_assert!(layout.row.typ == PanelRowLayoutType::DynamicRow);

      if layout.row.typ == PanelRowLayoutType::StaticRow
        || layout.row.typ == PanelRowLayoutType::DynamicRow
      {
        layout.row.item_width = 0f32;
        layout.row.item_offset = 0f32;
      }

      Some(())
    });
  }

  pub fn layout_row(&self, fmt: LayoutFormat, height: f32, ratio: &[f32]) {
    debug_assert!(self.current_win.borrow().is_some());

    self.current_win.borrow().as_ref().map(|winptr| {
      let win = winptr.borrow();
      let mut layout = win.layout.borrow_mut();

      self.panel_layout(&win, height, ratio.len() as i32);
      if fmt == LayoutFormat::Dynamic {
        // calculate width of undefined widget ratios
        layout.row.ratio = ratio.as_ptr();
        let (n_undef, r) = ratio.iter().fold((0i32, 0f32), |acc, r| {
          if *r < 0f32 {
            (acc.0 + 1, acc.1)
          } else {
            (acc.0, acc.1 + r)
          }
        });

        let r = saturate(1f32 - r);
        layout.row.typ = PanelRowLayoutType::Dynamic;
        layout.row.item_width = if r > 0f32 && n_undef > 0 {
          r / n_undef as f32
        } else {
          0f32
        };
      } else {
        layout.row.ratio = ratio.as_ptr();
        layout.row.typ = PanelRowLayoutType::Static;
        layout.row.item_width = 0f32;
      }

      layout.row.item_offset = 0f32;
      layout.row.filled = 0f32;

      Some(())
    });
  }

  pub fn layout_row_template_begin(&self, height: f32) {
    debug_assert!(self.current_win.borrow().is_some());

    self.current_win.borrow().as_ref().and_then(|winptr| {
      let win = winptr.borrow();
      self.panel_layout(&win, height, 1);

      let mut layout = win.layout.borrow_mut();
      layout.row.typ = PanelRowLayoutType::Template;
      layout.row.columns = 0;
      layout.row.ratio = std::ptr::null_mut();
      layout.row.item_width = 0f32;
      layout.row.item_height = 0f32;
      layout.row.item_offset = 0f32;
      layout.row.filled = 0f32;
      layout.row.item.x = 0f32;
      layout.row.item.y = 0f32;
      layout.row.item.w = 0f32;
      layout.row.item.h = 0f32;

      Some(())
    });
  }

  pub fn layout_row_template_push_dynamic(&self) {
    debug_assert!(self.current_win.borrow().is_some());

    self.current_win.borrow().as_ref().and_then(|winptr| {
      let win = winptr.borrow();
      let mut layout = win.layout.borrow_mut();

      debug_assert!(layout.row.typ == PanelRowLayoutType::Template);
      debug_assert!(
        layout.row.columns
          < crate::hmi::panel::MAX_LAYOUT_ROW_TEMPLATE_COLUMNS as i32
      );

      if layout.row.typ == PanelRowLayoutType::Template
        && layout.row.columns
          < crate::hmi::panel::MAX_LAYOUT_ROW_TEMPLATE_COLUMNS as i32
      {
        let idx = layout.row.columns as usize;
        layout.row.templates[idx] -= 1f32;
        layout.row.columns += 1;
      }

      Some(())
    });
  }

  pub fn layout_row_template_push_variable(&self, min_width: f32) {
    debug_assert!(self.current_win.borrow().is_some());

    self.current_win.borrow().as_ref().and_then(|winptr| {
      let win = winptr.borrow();
      let mut layout = win.layout.borrow_mut();

      debug_assert!(layout.row.typ == PanelRowLayoutType::Template);
      debug_assert!(
        layout.row.columns
          < crate::hmi::panel::MAX_LAYOUT_ROW_TEMPLATE_COLUMNS as i32
      );

      if layout.row.typ == PanelRowLayoutType::Template
        && layout.row.columns
          < crate::hmi::panel::MAX_LAYOUT_ROW_TEMPLATE_COLUMNS as i32
      {
        let idx = layout.row.columns as usize;
        layout.row.templates[idx] = -min_width;
        layout.row.columns += 1;
      }

      Some(())
    });
  }

  pub fn layout_row_template_push_static(&self, width: f32) {
    debug_assert!(self.current_win.borrow().is_some());

    self.current_win.borrow().as_ref().and_then(|winptr| {
      let win = winptr.borrow();
      let mut layout = win.layout.borrow_mut();

      debug_assert!(layout.row.typ == PanelRowLayoutType::Template);
      debug_assert!(
        layout.row.columns
          < crate::hmi::panel::MAX_LAYOUT_ROW_TEMPLATE_COLUMNS as i32
      );

      if layout.row.typ == PanelRowLayoutType::Template
        && layout.row.columns
          < crate::hmi::panel::MAX_LAYOUT_ROW_TEMPLATE_COLUMNS as i32
      {
        let idx = layout.row.columns as usize;
        layout.row.templates[idx] = width;
        layout.row.columns += 1;
      }

      Some(())
    });
  }

  pub fn layout_row_template_end(&self) {
    debug_assert!(self.current_win.borrow().is_some());

    self.current_win.borrow().as_ref().and_then(|winptr| {
      let win = winptr.borrow();
      let mut layout = win.layout.borrow_mut();

      debug_assert!(layout.row.typ == PanelRowLayoutType::Template);

      if layout.row.typ != PanelRowLayoutType::Template {
        return Some(());
      }

      // from 0 .. layout.row.columns
      let (
        variable_count,
        min_variable_count,
        min_fixed_width,
        total_fixed_width,
        max_variable_width,
      ) = (0 .. layout.row.columns).fold(
        (0, 0, 0f32, 0f32, 0f32),
        |(
          variable_count,
          min_variable_count,
          min_fixed_width,
          total_fixed_width,
          max_variable_width,
        ),
         idx| {
          let width = layout.row.templates[idx as usize];
          if width >= 0f32 {
            (
              variable_count,
              min_variable_count,
              min_fixed_width + width,
              total_fixed_width + width,
              max_variable_width,
            )
          } else if width < -1f32 {
            let width = -width;
            (
              variable_count + 1,
              min_variable_count,
              min_fixed_width,
              total_fixed_width + width,
              max_variable_width.max(width),
            )
          } else {
            (
              variable_count + 1,
              min_variable_count + 1,
              min_fixed_width,
              total_fixed_width,
              max_variable_width,
            )
          }
        },
      );

      if variable_count == 0 {
        return Some(());
      }

      let space = Self::layout_row_calculate_usable_space(
        &self.style,
        layout.typ,
        layout.bounds.w,
        layout.row.columns,
      );

      let var_width =
        (space - min_fixed_width).max(0f32) / variable_count as f32;
      let enough_space = var_width >= max_variable_width;
      let var_width = if !enough_space {
        (space - total_fixed_width).max(0f32) / min_variable_count as f32
      } else {
        var_width
      };

      (0 .. layout.row.columns).for_each(|idx| {
        let w = layout.row.templates[idx as usize];
        let w = if w >= 0f32 {
          w
        } else {
          if w < -1f32 && !enough_space {
            -w
          } else {
            var_width
          }
        };

        layout.row.templates[idx as usize] = w;
      });

      Some(())
    });
  }

  pub fn layout_space_begin(
    &self,
    fmt: LayoutFormat,
    height: f32,
    widget_count: i32,
  ) {
    debug_assert!(self.current_win.borrow().is_some());

    self.current_win.borrow().as_ref().and_then(|winptr| {
      let win = winptr.borrow();
      self.panel_layout(&win, height, widget_count);
      let mut layout = win.layout.borrow_mut();
      layout.row.typ = if fmt == LayoutFormat::Static {
        PanelRowLayoutType::StaticFree
      } else {
        PanelRowLayoutType::DynamicFree
      };

      layout.row.ratio = std::ptr::null_mut();
      layout.row.filled = 0f32;
      layout.row.item_width = 0f32;
      layout.row.item_offset = 0f32;

      Some(())
    });
  }

  pub fn layout_space_end(&self) {
    debug_assert!(self.current_win.borrow().is_some());

    self.current_win.borrow().as_ref().and_then(|winptr| {
      let win = winptr.borrow();
      let mut layout = win.layout.borrow_mut();

      layout.row.item_width = 0f32;
      layout.row.item_offset = 0f32;
      layout.row.item_height = 0f32;
      layout.row.item = RectangleF32::new(0f32, 0f32, 0f32, 0f32);

      Some(())
    });
  }

  pub fn layout_space_push(&self, rect: &RectangleF32) {
    debug_assert!(self.current_win.borrow().is_some());

    self.current_win.borrow().as_ref().and_then(|winptr| {
      let win = winptr.borrow();
      let mut layout = win.layout.borrow_mut();
      layout.row.item = *rect;

      Some(())
    });
  }

  pub fn layout_space_bounds(&self) -> RectangleF32 {
    debug_assert!(self.current_win.borrow().is_some());

    self.current_win.borrow().as_ref().map_or(
      RectangleF32::new(0f32, 0f32, 0f32, 0f32),
      |winptr| {
        let win = winptr.borrow();
        let layout = win.layout.borrow_mut();

        RectangleF32::new(
          layout.clip.x,
          layout.clip.y,
          layout.clip.w,
          layout.row.height,
        )
      },
    )
  }

  pub fn layout_widget_bounds(&self) -> RectangleF32 {
    debug_assert!(self.current_win.borrow().is_some());

    self.current_win.borrow().as_ref().map_or(
      RectangleF32::new(0f32, 0f32, 0f32, 0f32),
      |winptr| {
        let win = winptr.borrow();
        let layout = win.layout.borrow_mut();

        RectangleF32::new(
          layout.at_x,
          layout.at_y,
          layout.bounds.w - (layout.at_x - layout.bounds.x).max(0f32),
          layout.row.height,
        )
      },
    )
  }

  pub fn layout_space_to_screen(&self, ret: Vec2F32) -> Vec2F32 {
    debug_assert!(self.current_win.borrow().is_some());
    self
      .current_win
      .borrow()
      .as_ref()
      .map_or(Vec2F32::same(0f32), |winptr| {
        let win = winptr.borrow();
        let layout = win.layout.borrow_mut();

        Vec2F32::new(
          ret.x + layout.at_x - unsafe { *layout.offset_x as f32 },
          ret.y + layout.at_y - unsafe { *layout.offset_y as f32 },
        )
      })
  }

  pub fn layout_space_to_local(&self, ret: Vec2F32) -> Vec2F32 {
    debug_assert!(self.current_win.borrow().is_some());
    self
      .current_win
      .borrow()
      .as_ref()
      .map_or(Vec2F32::same(0f32), |winptr| {
        let win = winptr.borrow();
        let layout = win.layout.borrow_mut();

        Vec2F32::new(
          ret.x - layout.at_x + unsafe { *layout.offset_x as f32 },
          ret.y - layout.at_y + unsafe { *layout.offset_y as f32 },
        )
      })
  }

  pub fn layout_space_rect_to_screen(
    &self,
    ret: &RectangleF32,
  ) -> RectangleF32 {
    debug_assert!(self.current_win.borrow().is_some());
    self.current_win.borrow().as_ref().map_or(
      RectangleF32::new(0f32, 0f32, 0f32, 0f32),
      |winptr| {
        let win = winptr.borrow();
        let layout = win.layout.borrow_mut();

        RectangleF32::new(
          ret.x + layout.at_x - unsafe { *layout.offset_x as f32 },
          ret.y + layout.at_y - unsafe { *layout.offset_y as f32 },
          ret.w,
          ret.h,
        )
      },
    )
  }

  pub fn layout_space_rect_to_local(&self, ret: &RectangleF32) -> RectangleF32 {
    debug_assert!(self.current_win.borrow().is_some());
    self.current_win.borrow().as_ref().map_or(
      RectangleF32::new(0f32, 0f32, 0f32, 0f32),
      |winptr| {
        let win = winptr.borrow();
        let layout = win.layout.borrow_mut();

        RectangleF32::new(
          ret.x - layout.at_x + unsafe { *layout.offset_x as f32 },
          ret.y - layout.at_y + unsafe { *layout.offset_y as f32 },
          ret.w,
          ret.h,
        )
      },
    )
  }

  pub fn panel_alloc_row(&self, win: &Window) {
    let (row_height, num_columns) = {
      let spacing = self.style.window.spacing;
      let layout = win.layout.borrow();
      (layout.row.height - spacing.y, layout.row.columns)
    };

    self.panel_layout(win, row_height, num_columns)
  }

  pub fn layout_widget_space(
    &self,
    bounds: &RectangleF32,
    modify: bool,
  ) -> RectangleF32 {
    debug_assert!(self.current_win.borrow().is_some());

    self
      .current_win
      .borrow()
      .as_ref()
      .map_or(*bounds, |winptr| {
        let win = winptr.borrow();
        let mut layout = win.layout.borrow_mut();
        let mut bounds = *bounds;

        let spacing = self.style.window.spacing;
        let padding = self.style.get_panel_padding(layout.typ);
        let panel_space = Self::layout_row_calculate_usable_space(
          &self.style,
          layout.typ,
          layout.bounds.w,
          layout.row.columns,
        );

        struct ItemSpacingInfo {
          item_offset:  f32,
          item_width:   f32,
          item_spacing: f32,
        }

        let frac_fn = |x: f32| x - (x as i32) as f32;
        // calculate the width of one item inside the current layout space.

        match layout.row.typ {
          PanelRowLayoutType::DynamicFixed => {
            // scaling fixed size widgets item width
            let w = panel_space.max(1f32) / layout.row.columns as f32;
            let item_offset = layout.row.index as f32 * w;
            let item_width = w + frac_fn(item_offset);
            let item_spacing = layout.row.index as f32 + spacing.x;

            Some(ItemSpacingInfo {
              item_offset,
              item_spacing,
              item_width,
            })
          }

          PanelRowLayoutType::DynamicRow => {
            // scaling single ratio widget width
            let w = layout.row.item_width * panel_space;
            let item_offset = layout.row.item_offset;
            let item_width = w + frac_fn(item_offset);
            let item_spacing = 0f32;

            if modify {
              layout.row.item_offset += w + spacing.x;
              layout.row.filled += layout.row.item_width;
              layout.row.index = 0;
            }

            Some(ItemSpacingInfo {
              item_offset,
              item_spacing,
              item_width,
            })
          }

          PanelRowLayoutType::DynamicFree => {
            // free widget placing
            bounds.x = layout.at_x + (layout.bounds.w * layout.row.item.x);
            bounds.x -= unsafe { *layout.offset_x as f32 };
            bounds.y = layout.at_y + (layout.row.height * layout.row.item.y);
            bounds.y -= unsafe { *layout.offset_y as f32 };
            bounds.w = layout.bounds.w * layout.row.item.w + frac_fn(bounds.x);
            bounds.h =
              layout.row.height * layout.row.item.h + frac_fn(bounds.y);
            None
          }

          PanelRowLayoutType::Dynamic => {
            // scaling arrays of panel width rations for every widget
            assert!(layout.row.ratio != std::ptr::null_mut());
            let ratio = unsafe {
              let idx = layout.row.index as isize;
              if *layout.row.ratio.offset(idx) < 0f32 {
                layout.row.item_width
              } else {
                *layout.row.ratio.offset(idx)
              }
            };

            let w = ratio * panel_space;
            if modify {
              layout.row.item_offset += w;
              layout.row.filled += ratio;
            }

            Some(ItemSpacingInfo {
              item_spacing: layout.row.index as f32 * spacing.x,
              item_offset:  layout.row.item_offset,
              item_width:   w + frac_fn(layout.row.item_offset),
            })
          }

          PanelRowLayoutType::StaticFixed => {
            // non-scaling fixed widgets item width
            let item_width = layout.row.item_width;
            let item_offset = layout.row.index as f32 * item_width;
            let item_spacing = layout.row.index as f32 * spacing.x;

            Some(ItemSpacingInfo {
              item_width,
              item_offset,
              item_spacing,
            })
          }

          PanelRowLayoutType::StaticRow => {
            // scaling single ratio widget width
            let item_width = layout.row.item_width;
            let item_offset = layout.row.item_offset;
            let item_spacing = layout.row.index as f32 * spacing.x;
            if modify {
              layout.row.item_offset += item_width;
            }

            Some(ItemSpacingInfo {
              item_width,
              item_offset,
              item_spacing,
            })
          }

          PanelRowLayoutType::StaticFree => {
            // free widget placing
            bounds.x = layout.at_x + layout.row.item.x;
            bounds.w = layout.row.item.w;
            if (bounds.x + bounds.w) > layout.max_x && modify {
              layout.max_x = bounds.x + bounds.w;
            }
            bounds.x -= unsafe { *layout.offset_x as f32 };
            bounds.y = layout.at_y + layout.row.item.y;
            bounds.y -= unsafe { *layout.offset_y as f32 };
            bounds.h = layout.row.item.h;

            None
          }

          PanelRowLayoutType::Static => {
            // non-scaling array of panel pixel width for every widget
            let item_spacing = layout.row.index as f32 * spacing.x;
            let item_width = unsafe {
              let idx = layout.row.index as isize;
              *layout.row.ratio.offset(idx)
            };

            let item_offset = layout.row.item_offset;
            if modify {
              layout.row.item_offset += item_width;
            }

            Some(ItemSpacingInfo {
              item_spacing,
              item_width,
              item_offset,
            })
          }

          PanelRowLayoutType::Template => {
            // stretchy row layout with combined dynamic/static widget width
            assert!(layout.row.index < layout.row.columns);
            assert!(
              layout.row.index
                < crate::hmi::panel::MAX_LAYOUT_ROW_TEMPLATE_COLUMNS as i32
            );

            let w = layout.row.templates[layout.row.index as usize];

            let item_offset = layout.row.item_offset;
            let item_width = w + frac_fn(item_offset);
            let item_spacing = layout.row.index as f32 * spacing.x;
            if modify {
              layout.row.item_offset += w;
            }

            Some(ItemSpacingInfo {
              item_offset,
              item_width,
              item_spacing,
            })
          }

          _ => {
            debug_assert!(false, "No layout defined!");
            None
          }
        }
        .and_then(|spc| {
          bounds.w = spc.item_width;
          bounds.h = layout.row.height - spacing.y;
          bounds.y = layout.at_y - unsafe { *layout.offset_y as f32 };
          bounds.x =
            layout.at_x + spc.item_offset + spc.item_spacing + padding.x;

          if (bounds.x + bounds.w) > layout.max_x && modify {
            layout.max_x = bounds.x + bounds.w;
          }

          Some(())
        });

        bounds
      })
  }

  fn panel_alloc_space(&self, bounds: &mut RectangleF32) {
    debug_assert!(self.current_win.borrow().is_some());

    self.current_win.borrow().as_ref().map(|winptr| {
      // check if the end of the row was hit and begin a new row if true
      let win = winptr.borrow();
      let alloc_row = {
        let layout = win.layout.borrow();
        layout.row.index >= layout.row.columns
      };

      if alloc_row {
        self.panel_alloc_row(&win);
      }

      *bounds = self.layout_widget_space(&bounds, true);
      win.layout.borrow_mut().row.index += 1;
      Some(())
    });
  }

  fn layout_peek(&self, bounds: &mut RectangleF32) {
    debug_assert!(self.current_win.borrow().is_some());
    self.current_win.borrow().as_ref().map(|winptr| {
      let win = winptr.borrow();

      let (y, index) = {
        // make this go out of scope because it's mut borrowed by
        // layout_widget_space() below
        let mut layout = win.layout.borrow_mut();
        if layout.row.index >= layout.row.columns {
          layout.at_y += layout.row.height;
          layout.row.index = 0;
        }

        (layout.at_y, layout.row.index)
      };

      *bounds = self.layout_widget_space(&bounds, true);
      let mut layout = win.layout.borrow_mut();
      if layout.row.index == 0 {
        bounds.x -= layout.row.item_offset;
      }
      layout.at_y = y;
      layout.row.index = index;

      Some(())
    });
  }

  fn widget_bounds(&self) -> RectangleF32 {
    debug_assert!(self.current_win.borrow().is_some());
    self.current_win.borrow().as_ref().map_or(
      RectangleF32::new(0f32, 0f32, 0f32, 0f32),
      |_| {
        let mut bounds = RectangleF32::new(0f32, 0f32, 0f32, 0f32);
        self.layout_peek(&mut bounds);
        bounds
      },
    )
  }

  fn widget_position(&self) -> Vec2F32 {
    let bounds = self.widget_bounds();
    Vec2F32::new(bounds.x, bounds.y)
  }

  fn widget_size(&self) -> Vec2F32 {
    let bounds = self.widget_bounds();
    Vec2F32::new(bounds.x, bounds.y)
  }

  fn widget_width(&self) -> f32 {
    let bounds = self.widget_bounds();
    bounds.w
  }

  fn widget_height(&self) -> f32 {
    let bounds = self.widget_bounds();
    bounds.h
  }

  fn widget_is_hovered(&self) -> bool {
    debug_assert!(self.current_win.borrow().is_some());
    self.current_win.borrow().as_ref().map_or(false, |winptr| {
      let clip = winptr.borrow().layout.borrow().clip;
      let clip = RectangleF32::new(
        (clip.x as i32) as f32,
        (clip.y as i32) as f32,
        (clip.w as i32) as f32,
        (clip.h as i32) as f32,
      );

      let mut bounds = RectangleF32::new(0f32, 0f32, 0f32, 0f32);
      self.layout_peek(&mut bounds);

      if !clip.intersect(&bounds) {
        false
      } else {
        self.input.borrow().is_mouse_hovering_rect(&bounds)
      }
    })
  }

  fn widget_is_mouse_clicked(&self, btn: MouseButtonId) -> bool {
    debug_assert!(self.current_win.borrow().is_some());
    self.current_win.borrow().as_ref().map_or(false, |winptr| {
      let clip = winptr.borrow().layout.borrow().clip;
      let clip = RectangleF32::new(
        (clip.x as i32) as f32,
        (clip.y as i32) as f32,
        (clip.w as i32) as f32,
        (clip.h as i32) as f32,
      );

      let mut bounds = RectangleF32::new(0f32, 0f32, 0f32, 0f32);
      self.layout_peek(&mut bounds);

      if !clip.intersect(&bounds) {
        false
      } else {
        self.input.borrow().mouse_clicked(btn, &bounds)
      }
    })
  }

  fn widget_has_mouse_click_down(
    &self,
    btn: MouseButtonId,
    down: bool,
  ) -> bool {
    debug_assert!(self.current_win.borrow().is_some());
    self.current_win.borrow().as_ref().map_or(false, |winptr| {
      let clip = winptr.borrow().layout.borrow().clip;
      let clip = RectangleF32::new(
        (clip.x as i32) as f32,
        (clip.y as i32) as f32,
        (clip.w as i32) as f32,
        (clip.h as i32) as f32,
      );

      let mut bounds = RectangleF32::new(0f32, 0f32, 0f32, 0f32);
      self.layout_peek(&mut bounds);

      if !clip.intersect(&bounds) {
        false
      } else {
        self
          .input
          .borrow()
          .has_mouse_click_down_in_rect(btn, &bounds, down)
      }
    })
  }

  fn widget(
    &self,
    bounds: &RectangleF32,
  ) -> (WidgetLayoutStates, RectangleF32) {
    debug_assert!(self.current_win.borrow().is_some());

    self.current_win.borrow().as_ref().map_or(
      (
        WidgetLayoutStates::Invalid,
        RectangleF32::new(0f32, 0f32, 0f32, 0f32),
      ),
      |winptr| {
        let mut bounds = *bounds;

        // allocate space and check if the widget needs to be updated and drawn
        self.panel_alloc_space(&mut bounds);
        let win = winptr.borrow();
        let layout = win.layout.borrow();

        //  if one of these triggers you forgot to add an `if` condition around
        // either a window, group, popup, combobox or contextual menu
        // `begin` and `end` block. Example:
        // if (nk_begin(...) {...} nk_end(...); or
        // if (nk_group_begin(...) { nk_group_end(...);}
        debug_assert!(!(layout.flags.contains(PanelFlags::WindowMinimized)));
        debug_assert!(!(layout.flags.contains(PanelFlags::WindowHidden)));
        debug_assert!(!(layout.flags.contains(PanelFlags::WindowClosed)));

        // need to convert to int here to remove floating point errors
        bounds.x = (bounds.x as i32) as f32;
        bounds.y = (bounds.y as i32) as f32;
        bounds.w = (bounds.w as i32) as f32;
        bounds.h = (bounds.h as i32) as f32;

        let c = RectangleF32::new(
          (layout.clip.x as i32) as f32,
          (layout.clip.y as i32) as f32,
          (layout.clip.w as i32) as f32,
          (layout.clip.h as i32) as f32,
        );

        if !c.intersect(&bounds) {
          return (WidgetLayoutStates::Invalid, bounds);
        }

        let v = RectangleF32::union(&bounds, &c);
        if !v.contains_point(
          self.input.borrow().mouse.pos.x,
          self.input.borrow().mouse.pos.y,
        ) {
          return (WidgetLayoutStates::Rom, bounds);
        }

        (WidgetLayoutStates::Valid, bounds)
      },
    )
  }

  fn widget_fitting(
    &self,
    bounds: &RectangleF32,
    item_padding: Vec2F32,
  ) -> (WidgetLayoutStates, RectangleF32) {
    debug_assert!(self.current_win.borrow().is_some());

    self.current_win.borrow().as_ref().map_or(
      (
        WidgetLayoutStates::Invalid,
        RectangleF32::new(0f32, 0f32, 0f32, 0f32),
      ),
      |winptr| {
        // update the bounds to have no padding
        let (state, mut bounds) = self.widget(bounds);

        let win = winptr.borrow();
        let layout = win.layout.borrow();
        let panel_padding = self.style.get_panel_padding(layout.typ);
        if layout.row.index == 1 {
          bounds.w += panel_padding.x;
          bounds.x -= panel_padding.x;
        } else {
          bounds.x -= item_padding.x;
        }

        if layout.row.index == layout.row.columns {
          bounds.w += panel_padding.x;
        } else {
          bounds.w += item_padding.x;
        }

        (state, bounds)
      },
    )
  }

  fn spacing(&self, cols: i32) {
    debug_assert!(self.current_win.borrow().is_some());

    self.current_win.borrow().as_ref().and_then(|winptr| {
      // spacing over row boundaries
      let win = winptr.borrow();
      let (index, rows) = {
        let layout = win.layout.borrow();
        (
          (layout.row.index + cols) % layout.row.columns,
          (layout.row.index + cols) / layout.row.columns,
        )
      };

      let cols = if rows > 0 {
        (0 .. rows).for_each(|_| self.panel_alloc_row(&win));
        index
      } else {
        cols
      };

      // non table laout need to allocate space
      let layout_type = win.layout.borrow().row.typ;
      if layout_type != PanelRowLayoutType::DynamicFixed
        && layout_type != PanelRowLayoutType::StaticFixed
      {
        let mut none = RectangleF32::new(0f32, 0f32, 0f32, 0f32);
        (0 .. cols).for_each(|_| self.panel_alloc_space(&mut none));
      } else {
        win.layout.borrow_mut().row.index = index;
      }

      Some(())
    });
  }
}
