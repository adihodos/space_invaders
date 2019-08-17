use crate::{
  hmi::{
    base::{
      AntialiasingType, ButtonBehaviour, Consts, ConvertConfig, HashType,
      WidgetLayoutStates,
    },
    commands::CommandBuffer,
    input::{Input, MouseButtonId},
    panel::{LayoutFormat, Panel, PanelFlags, PanelRowLayoutType, PanelType},
    style::{ConfigurationStacks, Style, StyleItem},
    text_engine::Font,
    vertex_output::DrawList,
    window::Window,
  },
  math::{
    colors::RGBAColor,
    rectangle::RectangleF32,
    utility::{clamp, saturate},
    vec2::Vec2F32,
  },
};

use enumflags2::BitFlags;
use murmurhash64::murmur_hash64a;
use std::{cell::RefCell, rc::Rc};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CollapseStates {
  Minimized,
  Maximized,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShowStates {
  Hidden,
  Shown,
}

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

pub struct UiContext {
  pub input:             RefCell<Input>,
  pub style:             Style,
  pub last_widget_state: u32,
  pub button_behviour:   ButtonBehaviour,
  pub stacks:            ConfigurationStacks,
  pub delta_time_sec:    f32,
  draw_list:             DrawList,
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

impl<'a> UiContext {
  pub fn new(
    font: Font,
    config: ConvertConfig,
    line_aa: AntialiasingType,
    shape_aa: AntialiasingType,
  ) -> UiContext {
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

  fn find_window(&self, hash: HashType, name: &str) -> Option<WindowPtr> {
    self
      .windows
      .borrow()
      .iter()
      .find(|winptr| {
        let wnd = winptr.borrow();
        let res =
          wnd.id.borrow().name == hash && wnd.id.borrow().name_str == name;
        res
      })
      .and_then(|winptr| Some(Rc::clone(&winptr)))
  }

  fn insert_window(&self, win: WindowPtr, loc: WindowInsertLocation) {
    // check if not already inserted
    self
      .windows
      .borrow()
      .iter()
      .find(|winptr| {
        winptr.borrow().id.borrow().handle == win.borrow().id.borrow().handle
      })
      .map_or(
        Some(()), /* Return something so that we know that we must insert
                   * the window */
        |_| {
          // window already inserted, so do nothing
          None
        },
      )
      .and_then(|_| {
        let mut win_list = self.windows.borrow_mut();
        if win_list.is_empty() {
          win_list.push(win);
          return Some(());
        }

        win.borrow_mut().flags.remove(PanelFlags::WindowRom);

        match loc {
          WindowInsertLocation::Back => {
            // set ROM mode for the previous window
            win_list.last_mut().and_then(|last_wnd| {
              Some(last_wnd.borrow_mut().flags.insert(PanelFlags::WindowRom))
            });

            self.active_win.replace(Some(win.clone()));
            win_list.push(win);
          }

          WindowInsertLocation::Front => {
            win_list.insert(0, win);
          }
        }

        Some(())
      });
  }

  fn remove_window(&self, win: WindowPtr) {
    self
      .windows
      .borrow()
      .iter()
      .position(|winptr| *winptr.borrow() == *win.borrow())
      .and_then(|win_idx| {
        self.windows.borrow_mut().remove(win_idx);
        Some(())
      });

    self
      .active_win
      .borrow()
      .as_ref()
      .map_or(
        Some(()), // no active window yet
        |winptr| {
          // the window to be removed was the active window
          if *winptr.borrow() == *win.borrow() {
            Some(())
          } else {
            None
          }
        },
      )
      .and_then(|_| {
        // remove read-only from the last window
        // last window becomes active window
        let last_wnd =
          self.windows.borrow_mut().last_mut().and_then(|winptr| {
            winptr.borrow_mut().flags.remove(PanelFlags::WindowRom);
            Some(Rc::clone(winptr))
          });

        self.active_win.replace(last_wnd);
        Some(())
      });
  }

  pub fn begin(
    &mut self,
    title: &str,
    bounds: RectangleF32,
    flags: BitFlags<PanelFlags>,
  ) -> bool {
    self.begin_titled(title, title, bounds, flags)
  }

  pub fn begin_titled(
    &mut self,
    name: &str,
    title: &str,
    bounds: RectangleF32,
    flags: BitFlags<PanelFlags>,
  ) -> bool {
    debug_assert!(
      self.current_win.borrow().is_none(),
      "if this triggers you missed an end() call"
    );

    let winptr = self
      .find_window(murmur_hash64a(name.as_bytes(), 64), name)
      .map_or(None, |wndptr| {
        // existing window, needs updating
        let flags = {
          let mut f = wndptr.borrow().flags;
          f.remove(PanelFlags::WindowDynamic);
          f.insert(flags);

          if !f.contains(PanelFlags::WindowMovable | PanelFlags::WindowScalable)
          {
            wndptr.borrow().bounds.replace(bounds);
          }
          f
        };

        wndptr.borrow_mut().flags = flags;

        debug_assert!(
          wndptr.borrow().seq != self.seq,
          "If this triggers you either have more than one window with the \
           same name or you forgot to actually draw the window"
        );

        wndptr.borrow_mut().seq = self.seq;
        // no active window so set this as the active window
        if self.active_win.borrow().is_none()
          && !flags.contains(PanelFlags::WindowHidden)
        {
          self.active_win.borrow_mut().replace(Rc::clone(&wndptr));
        }

        Some(wndptr)
      })
      .map_or_else(
        || {
          // window does no exist, create it
          let wndptr = Rc::new(RefCell::new(Window::new(
            self.alloc_win_handle(),
            murmur_hash64a(name.as_bytes(), 64),
            name,
            flags,
            bounds,
          )));

          if flags.contains(PanelFlags::WindowBackground) {
            self.insert_window(Rc::clone(&wndptr), WindowInsertLocation::Front);
          } else {
            self.insert_window(Rc::clone(&wndptr), WindowInsertLocation::Back);
          }

          if self.active_win.borrow().is_none() {
            self.active_win.borrow_mut().replace(Rc::clone(&wndptr));
          }

          wndptr
        },
        |existing_wnd_ptr| existing_wnd_ptr,
      );

    if winptr.borrow().flags.contains(PanelFlags::WindowHidden) {
      self.current_win.borrow_mut().replace(winptr);
      return false;
    }

    // window overlapping
    self.do_window_overlapping(Rc::clone(&winptr));
    self.current_win.borrow_mut().replace(Rc::clone(&winptr));
    self.panel_begin(title, PanelType::Window.into())
  }

  fn find_window_index_by_handle(&self, handle: usize) -> Option<usize> {
    self
      .windows
      .borrow()
      .iter()
      .position(|winptr| winptr.borrow().id.borrow().handle == handle)
  }

  fn is_active_window(&self, wndptr: &WindowPtr) -> bool {
    self
      .active_win
      .borrow()
      .as_ref()
      .map_or(false, |active_win| {
        *active_win.borrow().id.borrow() == *wndptr.borrow().id.borrow()
      })
  }

  fn is_last_window(&self, wndptr: &WindowPtr) -> bool {
    self
      .windows
      .borrow()
      .last()
      .map_or(false, |last_wnd| *last_wnd.borrow() == *wndptr.borrow())
  }

  fn do_window_overlapping(&mut self, winptr: WindowPtr) {
    let flags = winptr.borrow().flags;

    if flags.contains(PanelFlags::WindowHidden)
      || flags.contains(PanelFlags::WindowNoInput)
    {
      return;
    }

    let h = self.style.font.scale
      + 2f32 * self.style.window.header.padding.y
      + 2f32 * self.style.window.header.label_padding.y;

    let win_bounds = if !flags.contains(PanelFlags::WindowMinimized) {
      *winptr.borrow().bounds.borrow()
    } else {
      RectangleF32 {
        h,
        ..*winptr.borrow().bounds.borrow()
      }
    };

    let inpanel = self.input.borrow().has_mouse_click_down_in_rect(
      MouseButtonId::ButtonLeft,
      &win_bounds,
      true,
    ) && self
      .input
      .borrow()
      .is_button_clicked(MouseButtonId::ButtonLeft);

    // activate window if hovered and no other window is overlapping this window
    if !self.is_active_window(&winptr)
      && self.input.borrow().is_mouse_hovering_rect(&win_bounds)
      && !self.input.borrow().is_mouse_down(MouseButtonId::ButtonLeft)
    {
      self
        .find_window_index_by_handle(winptr.borrow().id.borrow().handle)
        .and_then(|idx| {
          if self.windows.borrow().len() >= (idx + 1) {
            return None;
          }

          let iter = self.windows.borrow()[idx + 1 ..]
            .iter()
            .find(|itr| {
              let iter_flags = itr.borrow().flags;

              let iter_bounds =
                if !iter_flags.contains(PanelFlags::WindowMinimized) {
                  *itr.borrow().bounds.borrow()
                } else {
                  RectangleF32 {
                    h,
                    ..*itr.borrow().bounds.borrow()
                  }
                };

              if iter_bounds.intersect(&win_bounds)
                && !iter_flags.contains(PanelFlags::WindowHidden)
              {
                return true;
              }

              let res = itr.borrow().popup.active
                && !iter_flags.contains(PanelFlags::WindowHidden)
                && itr.borrow().popup.win.as_ref().map_or(false, |popup_win| {
                  win_bounds.intersect(&popup_win.borrow().bounds())
                });

              res
            })
            .map(|wp| Rc::clone(wp));

          // activate window if clicked
          let iter = iter.and_then(|win| {
            if !inpanel || self.is_last_window(&winptr) {
              return None;
            }
            // try to find a panel with higher priority in the same position
            self
              .find_window_index_by_handle(win.borrow().id.borrow().handle)
              .and_then(|idx| {
                let window_list = self.windows.borrow();

                if window_list.len() >= (idx + 1) {
                  return None;
                }

                window_list[idx + 1 ..]
                  .iter()
                  .find(|iter| {
                    let iter_flags = iter.borrow().flags;
                    let iter_bounds =
                      if !iter_flags.contains(PanelFlags::WindowMinimized) {
                        *iter.borrow().bounds.borrow()
                      } else {
                        RectangleF32 {
                          h,
                          ..*iter.borrow().bounds.borrow()
                        }
                      };

                    let mouse_pos = self.input.borrow().mouse.pos;
                    if iter_bounds.contains_point(mouse_pos.x, mouse_pos.y)
                      && !iter_flags.contains(PanelFlags::WindowHidden)
                    {
                      return true;
                    }

                    let res = iter.borrow().popup.active
                      && !iter_flags.contains(PanelFlags::WindowHidden)
                      && iter.borrow().popup.win.as_ref().map_or(
                        false,
                        |popup_win| {
                          win_bounds.intersect(&popup_win.borrow().bounds())
                        },
                      );

                    res
                  })
                  .map(|wp| Rc::clone(wp))
              })
          });

          if iter.is_some()
            && !flags.contains(PanelFlags::WindowRom)
            && flags.contains(PanelFlags::WindowBackground)
          {
            winptr.borrow_mut().flags.insert(PanelFlags::WindowRom);
            let iter = iter.unwrap();
            iter.borrow_mut().flags.remove(PanelFlags::WindowRom);
            self.active_win.borrow_mut().replace(Rc::clone(&iter));
            if !iter.borrow().flags.contains(PanelFlags::WindowBackground) {
              // current window is active in that position so transfer to top
              // at the highest priority in stack
              self.remove_window(Rc::clone(&iter));
              self.insert_window(iter, WindowInsertLocation::Back);
            }
          } else {
            if iter.is_none() && !self.is_last_window(&winptr) {
              if !winptr.borrow().flags.contains(PanelFlags::WindowBackground) {
                // current window is active in that position so transfer to top
                // at the highest priority in stack
                self.remove_window(Rc::clone(&winptr));
                self.insert_window(
                  Rc::clone(&winptr),
                  WindowInsertLocation::Back,
                );
              }

              winptr.borrow_mut().flags.remove(PanelFlags::WindowRom);
              self.active_win.borrow_mut().replace(Rc::clone(&winptr));
            }

            if !self.is_last_window(&winptr)
              && !winptr.borrow().flags.contains(PanelFlags::WindowBackground)
            {
              winptr.borrow_mut().flags.insert(PanelFlags::WindowRom);
            }
          }

          Some(())
        });
    }
  }

  pub fn end(&mut self) {
    debug_assert!(
      self.current_win.borrow().is_some(),
      "If this triggers you forgot to call begin()"
    );

    let end_panel =
      self.current_win.borrow().as_ref().map_or(true, |curr_win| {
        curr_win.borrow().flags.contains(PanelFlags::WindowHidden)
          && curr_win.borrow().layout.borrow().typ == PanelType::Window
      });

    if end_panel {
      self.panel_end();
    }

    *self.current_win.borrow_mut() = None;
  }

  pub fn window_get_bounds(&self) -> RectangleF32 {
    debug_assert!(self.current_win.borrow().is_some());
    self
      .current_win
      .borrow()
      .as_ref()
      .map_or(RectangleF32::new(0f32, 0f32, 0f32, 0f32), |curr_win| {
        *curr_win.borrow().bounds.borrow()
      })
  }

  pub fn window_get_position(&self) -> Vec2F32 {
    let bounds = self.window_get_bounds();
    Vec2F32::new(bounds.x, bounds.y)
  }

  pub fn window_get_size(&self) -> Vec2F32 {
    let bounds = self.window_get_bounds();
    Vec2F32::new(bounds.w, bounds.h)
  }

  pub fn window_get_width(&self) -> f32 {
    self.window_get_bounds().w
  }

  pub fn window_get_height(&self) -> f32 {
    self.window_get_bounds().h
  }

  pub fn window_get_content_region(&self) -> RectangleF32 {
    debug_assert!(self.current_win.borrow().is_some());
    self
      .current_win
      .borrow()
      .as_ref()
      .map_or(RectangleF32::new(0f32, 0f32, 0f32, 0f32), |curr_win| {
        curr_win.borrow().layout.borrow().clip
      })
  }

  pub fn window_get_content_region_min(&self) -> Vec2F32 {
    let content_region = self.window_get_content_region();
    Vec2F32::new(content_region.x, content_region.y)
  }

  pub fn window_get_content_region_max(&self) -> Vec2F32 {
    let content_rect = self.window_get_content_region();
    Vec2F32 {
      x: content_rect.x + content_rect.w,
      y: content_rect.y + content_rect.h,
    }
  }

  pub fn window_get_content_region_size(&self) -> Vec2F32 {
    let content_region = self.window_get_content_region();
    Vec2F32 {
      x: content_region.w,
      y: content_region.h,
    }
  }

  pub fn window_has_focus(&self) -> bool {
    debug_assert!(self.current_win.borrow().is_some());
    self
      .current_win
      .borrow()
      .as_ref()
      .map_or(false, |curr_win| {
        self
          .active_win
          .borrow()
          .as_ref()
          .map_or(false, |active_win| {
            *curr_win.borrow() == *active_win.borrow()
          })
      })
  }

  pub fn window_is_hovered(&self) -> bool {
    debug_assert!(self.current_win.borrow().is_some());
    self
      .current_win
      .borrow()
      .as_ref()
      .map_or(false, |curr_win| {
        if curr_win.borrow().flags.contains(PanelFlags::WindowHidden) {
          return false;
        }

        self
          .input
          .borrow()
          .is_mouse_hovering_rect(&curr_win.borrow().bounds.borrow())
      })
  }

  pub fn window_is_any_hovered(&self) -> bool {
    self.windows.borrow().iter().any(|winptr| {
      let win = winptr.borrow();
      // check if window is hovered
      if win.flags.contains(PanelFlags::WindowHidden) {
        return false;
      }

      // check if popup is hovered
      let popup_hovered = win.popup.active
        && win.popup.win.as_ref().map_or(false, |popup_win| {
          self
            .input
            .borrow()
            .is_mouse_hovering_rect(&popup_win.borrow().bounds.borrow())
        });

      if popup_hovered {
        return true;
      }

      if win.flags.contains(PanelFlags::WindowMinimized) {
        let header = RectangleF32 {
          h: self.style.font.scale + 2f32 * self.style.window.header.padding.y,
          ..*win.bounds.borrow()
        };

        self.input.borrow().is_mouse_hovering_rect(&header)
      } else if self
        .input
        .borrow()
        .is_mouse_hovering_rect(&win.bounds.borrow())
      {
        true
      } else {
        false
      }
    })
  }

  pub fn window_is_collapsed(&self, name: &str) -> bool {
    self
      .find_window(murmur_hash64a(name.as_bytes(), 64), name)
      .map_or(false, |win| {
        win.borrow().flags.contains(PanelFlags::WindowMinimized)
      })
  }

  pub fn window_is_closed(&self, name: &str) -> bool {
    self
      .find_window(murmur_hash64a(name.as_bytes(), 64), name)
      .map_or(true, |win| {
        win.borrow().flags.contains(PanelFlags::WindowClosed)
      })
  }

  pub fn window_is_hidden(&self, name: &str) -> bool {
    self
      .find_window(murmur_hash64a(name.as_bytes(), 64), name)
      .map_or(true, |win| {
        win.borrow().flags.contains(PanelFlags::WindowHidden)
      })
  }

  pub fn window_is_active(&self, name: &str) -> bool {
    self
      .find_window(murmur_hash64a(name.as_bytes(), 64), name)
      .map_or(false, |win| self.is_active_window(&win))
  }

  pub fn window_find(&self, name: &str) -> Option<WindowPtr> {
    self.find_window(murmur_hash64a(name.as_bytes(), 64), name)
  }

  pub fn window_close(&mut self, name: &str) {
    self.window_find(name).and_then(|wnd| {
      debug_assert!(
        !self.is_active_window(&wnd),
        "Cannot close the currently active window!"
      );
      if !self.is_active_window(&wnd) {
        wnd
          .borrow_mut()
          .flags
          .insert(PanelFlags::WindowHidden | PanelFlags::WindowClosed);
      }

      Some(())
    });
  }

  pub fn window_set_bounds(&mut self, name: &str, bounds: RectangleF32) {
    self.window_find(name).and_then(|wnd| {
      debug_assert!(
        !self.is_active_window(&wnd),
        "Cannot close the currently active window!"
      );
      if !self.is_active_window(&wnd) {
        *wnd.borrow().bounds.borrow_mut() = bounds;
      }

      Some(())
    });
  }

  pub fn window_set_position(&mut self, name: &str, pos: Vec2F32) {
    self.window_find(name).and_then(|win| {
      let win = win.borrow();
      let mut bounds = win.bounds.borrow_mut();
      bounds.x = pos.x;
      bounds.y = pos.y;

      Some(())
    });
  }

  pub fn window_set_size(&mut self, name: &str, size: Vec2F32) {
    self.window_find(name).and_then(|win| {
      let win = win.borrow();
      let mut bounds = win.bounds.borrow_mut();
      bounds.w = size.x;
      bounds.h = size.y;

      Some(())
    });
  }

  pub fn window_collapse(&mut self, name: &str, collapse: CollapseStates) {
    self.window_find(name).and_then(|win| {
      if collapse == CollapseStates::Minimized {
        win.borrow_mut().flags.insert(PanelFlags::WindowMinimized);
      } else {
        win.borrow_mut().flags.remove(PanelFlags::WindowMinimized);
      }

      Some(())
    });
  }

  pub fn window_collapse_if<F: FnOnce() -> CollapseStates>(
    &mut self,
    name: &str,
    condition: F,
  ) {
    self.window_collapse(name, condition());
  }

  pub fn window_show(&mut self, name: &str, s: ShowStates) {
    self.window_find(name).and_then(|win| {
      match s {
        ShowStates::Hidden => {
          win.borrow_mut().flags.insert(PanelFlags::WindowHidden);
        }
        ShowStates::Shown => {
          win.borrow_mut().flags.remove(PanelFlags::WindowHidden);
        }
      }

      Some(())
    });
  }

  pub fn window_show_if<F: FnOnce() -> ShowStates>(
    &mut self,
    name: &str,
    show_cond: F,
  ) {
    self.window_show(name, show_cond());
  }

  pub fn window_set_focus(&mut self, name: &str) {
    let win = self.window_find(name);

    win
      .as_ref()
      .filter(|winptr| !self.is_last_window(&winptr))
      .and_then(|winptr| {
        self.remove_window(Rc::clone(&winptr));
        self.insert_window(Rc::clone(&winptr), WindowInsertLocation::Back);
        Some(())
      });

    self.active_win.replace(win);
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
    winptr.borrow_mut().layout = Box::new(RefCell::new(Panel::new(
      Rc::clone(&winptr.borrow().scroll),
      panel_type,
    )));

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
      let mut header = *winptr.borrow().bounds.borrow();
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
        let win = winptr.borrow();
        let mut bounds = win.bounds.borrow_mut();
        bounds.x += self.input.borrow().mouse.delta.x;
        bounds.y += self.input.borrow().mouse.delta.y;

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
      layout.bounds = *winptr.borrow().bounds.borrow();
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
      let mut header = *win.bounds.borrow();
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
        .map_or(false, |active_win| *active_win.borrow() == *win);

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
      let bounds = win.bounds.borrow();
      let body = RectangleF32 {
        y: bounds.y + layout.header_height,
        h: bounds.h - layout.header_height,
        ..*bounds
      };

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
        let winptr = win.clone();
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
            ..*win.bounds.borrow()
          };
          win.buffer.borrow_mut().fill_rect(
            empty_space,
            0f32,
            self.style.window.background,
          );

          // fill left empty space
          let empty_space = RectangleF32 {
            x: win.bounds.borrow().x,
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
          let adjust_for_scrollbar = if layout.offsets.borrow().scrollbar.y == 0
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
              ..*win.bounds.borrow()
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
            self.style.window.border
              + win.bounds.borrow().y
              + layout.header_height
          } else {
            if layout.flags.contains(PanelFlags::WindowDynamic) {
              layout.bounds.y + layout.bounds.h + layout.footer_height
            } else {
              win.bounds.borrow().y + win.bounds.borrow().h
            }
          };

          let border = RectangleF32 {
            h: padding_y - win.bounds.borrow().y,
            ..*win.bounds.borrow()
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
            let mut scaler = scaler;
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

            let mut win_bounds = win.bounds.borrow_mut();

            if left_mouse_down && left_mouse_click_in_scaler {
              let delta_x =
                if layout.flags.contains(PanelFlags::WindowScaleLeft) {
                  win_bounds.x += self.input.borrow().mouse.delta.x;
                  -self.input.borrow().mouse.delta.x
                } else {
                  self.input.borrow().mouse.delta.x
                };

              let window_size = self.style.window.min_size;

              // dragging in x-direction
              if (win_bounds.w + delta_x) >= window_size.x {
                if delta_x < 0f32
                  || (delta_x > 0f32
                    && self.input.borrow().mouse.pos.x >= scaler.x)
                {
                  win_bounds.w += delta_x;
                  scaler.x += self.input.borrow().mouse.delta.x;
                }
              }

              // dragging in y-direction (only possible if static window)
              if !layout.flags.contains(PanelFlags::WindowDynamic) {
                let inp = self.input.borrow();
                if window_size.y < win_bounds.h + inp.mouse.delta.y {
                  if inp.mouse.delta.y < 0f32
                    || (inp.mouse.delta.y > 0f32 && inp.mouse.pos.y >= scaler.y)
                  {
                    win_bounds.h += inp.mouse.delta.y;
                    scaler.y += inp.mouse.delta.y;
                  }
                }
              }

              // TODO : fix cursor!
              // ctx->style.cursor_active =
              // ctx->style.cursors[NK_CURSOR_RESIZE_TOP_RIGHT_DOWN_LEFT];
              self.input.borrow_mut().mouse.buttons
                [MouseButtonId::ButtonLeft as usize]
                .clicked_pos = Vec2F32::new(
                scaler.x + scaler.w / 2f32,
                scaler.y + scaler.h / 2f32,
              );
            }
          }
        }

        if !layout.is_sub() {
          // window is hidden so clear command buffer
          if layout.flags.contains(PanelFlags::WindowHidden) {
            win.buffer.borrow_mut().clear();
          }
        }

        // remove window read only mode flag was set so remove read only mode
        if layout.flags.contains(PanelFlags::WindowRemoveRom) {
          layout
            .flags
            .remove(PanelFlags::WindowRom | PanelFlags::WindowRemoveRom);
        }

        Some((winptr, layout.flags))
      })
      .and_then(|(winptr, win_flags)| {
        winptr.borrow_mut().flags = win_flags;
        // TODO: properties fix

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
      let bk = RectangleF32 {
        y: layout.at_y - 1f32,
        h: layout.row.height + 1f32,
        ..*win.bounds.borrow()
      };

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
      clamp(0f32, pixel_width / winptr.borrow().bounds.borrow().x, 1f32)
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
        let layout = win.layout.borrow();

        let res = Vec2F32::new(
          ret.x + layout.at_x - layout.offsets.borrow().scrollbar.x as f32,
          ret.y + layout.at_y - layout.offsets.borrow().scrollbar.y as f32,
        );

        res
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
        let layout = win.layout.borrow();

        let res = Vec2F32::new(
          ret.x - layout.at_x + layout.offsets.borrow().scrollbar.x as f32,
          ret.y - layout.at_y + layout.offsets.borrow().scrollbar.y as f32,
        );

        res
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
        let layout = win.layout.borrow();

        let res = RectangleF32::new(
          ret.x + layout.at_x - layout.offsets.borrow().scrollbar.x as f32,
          ret.y + layout.at_y - layout.offsets.borrow().scrollbar.y as f32,
          ret.w,
          ret.h,
        );

        res
      },
    )
  }

  pub fn layout_space_rect_to_local(&self, ret: &RectangleF32) -> RectangleF32 {
    debug_assert!(self.current_win.borrow().is_some());
    self.current_win.borrow().as_ref().map_or(
      RectangleF32::new(0f32, 0f32, 0f32, 0f32),
      |winptr| {
        let win = winptr.borrow();
        let layout = win.layout.borrow();

        let res = RectangleF32::new(
          ret.x - layout.at_x + layout.offsets.borrow().scrollbar.x as f32,
          ret.y - layout.at_y + layout.offsets.borrow().scrollbar.y as f32,
          ret.w,
          ret.h,
        );

        res
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
            bounds.x -= layout.offsets.borrow().scrollbar.x as f32;
            bounds.y = layout.at_y + (layout.row.height * layout.row.item.y);
            bounds.y -= layout.offsets.borrow().scrollbar.y as f32;
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
            bounds.x -= layout.offsets.borrow().scrollbar.x as f32;
            bounds.y = layout.at_y + layout.row.item.y;
            bounds.y -= layout.offsets.borrow().scrollbar.y as f32;
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
          bounds.y = layout.at_y - layout.offsets.borrow().scrollbar.y as f32;
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
