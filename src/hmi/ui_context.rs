use crate::{
  hmi::{
    base::{
      AntialiasingType, ButtonBehaviour, ConvertConfig, GenericHandle, HashType,
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
      .contains(PanelFlags::WindowHidden | PanelFlags::WindowMinimized)
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
      .and_then(|winptr| Some(winptr.clone()))
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

  // pub fn layout_space_rect_to_screen(&self, )
}
