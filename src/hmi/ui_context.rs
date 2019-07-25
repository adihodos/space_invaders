use crate::{
  hmi::{
    base::{
      AntialiasingType, ButtonBehaviour, ConvertConfig, GenericHandle, HashType,
    },
    commands::CommandBuffer,
    input::{Input, MouseButtonId},
    panel::{Panel, PanelFlags, PanelType},
    style::{ConfigurationStacks, Style},
    text_engine::Font,
    vertex_output::{DrawCommand, DrawIndexType, DrawList},
    window::Window,
  },
  math::{rectangle::RectangleF32, vertex_types::VertexPTC},
};

use enumflags2::BitFlags;
use murmurhash64::murmur_hash64a;
use num::ToPrimitive;

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

pub struct UiContext<'a> {
  pub input:             Input,
  pub style:             Style,
  pub last_widget_state: u32,
  pub button_behviour:   ButtonBehaviour,
  pub stacks:            ConfigurationStacks,
  pub delta_time_sec:    f32,
  draw_list:             DrawList<'a>,
  // TODO: text edit support
  overlay: CommandBuffer,
  // windows
  build:          i32,
  window_list:    Vec<Window>,
  active:         Option<usize>,
  current:        Option<usize>,
  count:          u32,
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
      input:             Input::new(),
      style:             Style::new(font),
      last_widget_state: 0,
      button_behviour:   ButtonBehaviour::default(),
      stacks:            ConfigurationStacks::default(),
      delta_time_sec:    0f32,
      draw_list:         DrawList::new(config, line_aa, shape_aa),
      overlay:           CommandBuffer::new(
        Some(RectangleF32::new(
          -8192_f32, -8192_f32, 16834_f32, 16834_f32,
        )),
        128,
      ),
      build:             0,
      window_list:       vec![],
      active:            None,
      current:           None,
      count:             0,
      seq:               0,
      win_handle_seq:    0,
    }
  }

  fn is_active_window(&self, idx: usize) -> bool {
    self.active.map_or(false, |active_idx| active_idx == idx)
  }

  fn is_current_window(&self, handle: usize) -> bool {
    self.current.map_or(false, |curr_wnd| curr_wnd == handle)
  }

  fn get_window_by_handle(&self, handle: usize) -> Option<&Window> {
    self.window_list.iter().find(|wnd| wnd.handle == handle)
  }

  fn get_window_mut_by_handle(&mut self, handle: usize) -> Option<&mut Window> {
    self.window_list.iter_mut().find(|wnd| wnd.handle == handle)
  }

  fn get_window_by_index(&self, idx: usize) -> Option<&Window> {
    if idx < self.window_list.len() {
      Some(&self.window_list[idx])
    } else {
      None
    }
  }

  fn get_window_mut_by_index(&mut self, idx: usize) -> Option<&mut Window> {
    if idx < self.window_list.len() {
      Some(&mut self.window_list[idx])
    } else {
      None
    }
  }

  fn get_current_window(&self) -> Option<&Window> {
    self
      .current
      .as_ref()
      .copied()
      .and_then(|curr_win_handle| self.get_window_by_handle(curr_win_handle))
  }

  fn get_current_window_mut(&mut self) -> Option<&mut Window> {
    self.current.as_ref().copied().and_then(|curr_win_handle| {
      // self.window_list.iter_mut().find(|wnd| wnd.handle == curr_win_handle)
      None
    })
  }

  pub fn clear(&mut self) {
    self.build = 0;
    self.last_widget_state = 0;
    self.style.cursor_active = 0; // FIX this
    self.overlay.clear();

    let mut wnd_list = std::mem::replace(&mut self.window_list, vec![]);
    let mut free_wnds = vec![];
    let mut remove_wnds = vec![];
    let mut new_active_wnd_idx = None;

    for (idx, win) in wnd_list.iter_mut().enumerate() {
      // make sure valid windows do not get removed
      if win.flags.contains(PanelFlags::WindowMinimized)
        && !win.flags.contains(PanelFlags::WindowClosed)
        && win.seq == self.seq
      {
        continue;
      }

      // remove hotness from hidden or closed windows
      if win
        .flags
        .contains(PanelFlags::WindowHidden | PanelFlags::WindowClosed)
        && self.is_active_window(win.handle)
      {
        new_active_wnd_idx = if idx > 0 { Some(idx - 1) } else { None };
      }

      // free unused popup windows
      win.popup.win.as_ref().copied().and_then(|popup_id| {
        // mark this popup window for destruction
        Some(free_wnds.push(popup_id))
      });

      // remove unused window state tables - TBD

      // window itself is not used anymore so free it
      if win.seq != self.seq || win.flags.contains(PanelFlags::WindowClosed) {
        // remove this window
        remove_wnds.push(win.handle);
        free_wnds.push(win.handle);
      }
    }

    std::mem::replace(&mut self.window_list, wnd_list);

    // one of the closed windows was the active window, previous window (if any)
    // becomes the active window
    new_active_wnd_idx.and_then(|wnd_idx| {
      // update active window and remove ROM flag from it
      self.window_list[wnd_idx]
        .flags
        .toggle(PanelFlags::WindowRom);
      Some(self.active = Some(self.window_list[wnd_idx].handle))
    });

    // remove all windows marked for removal
    remove_wnds
      .iter()
      .for_each(|wnd_handle| self.remove_window(*wnd_handle));

    self.seq += 1;
  }

  /// Removes a window from the list of existing windows.
  fn remove_window(&mut self, removed_win_handle: usize) {
    // remove window from list of windows
    self
      .window_list
      .iter()
      .position(|wnd| wnd.handle == removed_win_handle)
      .map(|idx| Some(self.window_list.remove(idx)))
      .expect("Window to be removed not found in window list !");

    // if no window active or window to be removed is the active window
    // update the active window to be the last window
    self
      .active
      .as_ref()
      .map_or(Some(()), |active_wnd| {
        if *active_wnd == removed_win_handle {
          Some(())
        } else {
          None
        }
      })
      .map(|_| {
        self.active = self.window_list.last_mut().and_then(|last_win| {
          // remove the ROM flag
          last_win.flags.toggle(PanelFlags::WindowRom);
          Some(last_win.handle)
        });
        Some(())
      });
  }

  fn alloc_win_handle(&mut self) -> usize {
    let handle = self.win_handle_seq;
    self.win_handle_seq += 1;
    handle
  }

  /// Returns the position of the searched window in the window list, if
  /// present.
  fn find_window(&self, hash: HashType, name: &str) -> Option<usize> {
    self.window_list.iter().position(|wnd| {
      wnd.name == hash && wnd.name_str.eq_ignore_ascii_case(name)
    })
  }

  fn find_window_index_by_handle(&self, handle: usize) -> Option<usize> {
    self.window_list.iter().position(|wnd| wnd.handle == handle)
  }

  fn insert_window(&mut self, win: Window, loc: WindowInsertLocation) -> usize {
    match loc {
      WindowInsertLocation::Back => {
        // set ROM mod for the last window
        self
          .window_list
          .last_mut()
          .and_then(|prev| Some(prev.flags.insert(PanelFlags::WindowRom)));
        // newly inserted window becomes the active window
        self.active = Some(win.handle);
        self.window_list.push(win);
        // remove ROM mode for the inserted window
        self.window_list.last_mut().and_then(|last_win| {
          Some(last_win.flags.toggle(PanelFlags::WindowRom))
        });
        self.window_list.len() - 1
      }

      WindowInsertLocation::Front => {
        self.window_list.insert(0, win);
        self.window_list.first_mut().and_then(|fst_win| {
          Some(fst_win.flags.toggle(PanelFlags::WindowRom))
        });
        0usize
      }
    }
  }

  /// Extended window start with separated title and identifier to allow
  /// multiple windows with same title but not name
  /// Returns true if the window can be filled up with widgets from this point
  /// until end() or false otherwise for example if minimized
  pub fn begin_titled(
    &mut self,
    name: &str,
    title: &str,
    bounds: RectangleF32,
    flags: BitFlags<PanelFlags>,
  ) -> bool {
    assert!(
      self.current.is_none(),
      "if this triggers you forgot a call to end()"
    );
    if self.current.is_some() {
      return false;
    }

    // find or create window
    let name_hash = murmur_hash64a(
      name.as_bytes(),
      PanelFlags::WindowTitle.to_u64().unwrap(),
    );

    let (mut win_idx, update_win) =
      self.find_window(name_hash, name).map_or_else(
        || {
          // window does not exist, create new window
          let win = Window::new(
            self.alloc_win_handle(),
            name_hash,
            title,
            flags,
            bounds,
          );

          let win_idx = if flags.contains(PanelFlags::WindowBackground) {
            self.insert_window(win, WindowInsertLocation::Front)
          } else {
            self.insert_window(win, WindowInsertLocation::Back)
          };

          (win_idx, false)
        },
        |win_idx| {
          // update existing window
          (win_idx, true)
        },
      );

    // Trigger update for existing window. Should be in the map_or_else above
    // but the borrow checker does not like that self is accessed in both
    // closures.
    if update_win {
      let wnd = &mut self.window_list[win_idx];
      wnd.flags.toggle(PanelFlags::WindowDynamic);
      wnd.flags.insert(flags);
      if !wnd
        .flags
        .contains(PanelFlags::WindowMovable | PanelFlags::WindowScalable)
      {
        wnd.bounds = bounds;
      }

      assert!(
        wnd.seq != self.seq,
        "If this assert triggers you either: have more than one window with \
         the same name or you forgot to actually draw the window (did not \
         call clear() on the context)"
      );

      wnd.seq = self.seq;
      if !wnd.flags.contains(PanelFlags::WindowHidden) && self.active.is_none()
      {
        self.active = Some(wnd.handle);
      }
    }

    if self.window_list[win_idx]
      .flags
      .contains(PanelFlags::WindowHidden)
    {
      self.window_list[win_idx].layout = None;
      self.current = Some(self.window_list[win_idx].handle);
      return false;
    }

    // clear the window's command buffer
    self.window_list[win_idx].buffer.clear();

    // window overlapping
    if !self.window_list[win_idx]
      .flags
      .contains(PanelFlags::WindowHidden)
      && !self.window_list[win_idx]
        .flags
        .contains(PanelFlags::WindowNoInput)
    {
      let h = self.style.font.scale
        + 2f32
        + self.style.window.header.padding.y
        + (2f32 * self.style.window.header.label_padding.y);

      let win_bounds = if !self.window_list[win_idx]
        .flags
        .contains(PanelFlags::WindowMinimized)
      {
        self.window_list[win_idx].bounds
      } else {
        let rect = self.window_list[win_idx].bounds;
        RectangleF32::new(rect.x, rect.y, rect.w, h)
      };

      // activate window if hovered and no other window is overlapping this
      // window
      let in_panel = self.input.has_mouse_click_down_in_rect(
        MouseButtonId::ButtonLeft,
        &win_bounds,
        true,
      );

      let is_hovered = self.input.is_mouse_hovering_rect(&win_bounds);

      if !self.is_active_window(self.window_list[win_idx].handle)
        && is_hovered
        && !self.input.has_mouse_down(MouseButtonId::ButtonLeft)
      {
        for this_wnd_idx in (win_idx + 1) .. self.window_list.len() {
          let bounds = if !self.window_list[this_wnd_idx]
            .flags
            .contains(PanelFlags::WindowMinimized)
          {
            self.window_list[this_wnd_idx].bounds
          } else {
            let bounds = self.window_list[this_wnd_idx].bounds;
            RectangleF32::new(bounds.x, bounds.y, bounds.w, h)
          };

          let is_hidden = self.window_list[this_wnd_idx]
            .flags
            .contains(PanelFlags::WindowHidden);

          if win_bounds.intersect(&bounds) && !is_hidden {
            win_idx = this_wnd_idx;
            break;
          }

          let has_active_popup =
            self.window_list[this_wnd_idx].popup.win.is_some()
              && self.window_list[this_wnd_idx].popup.active;

          if !has_active_popup {
            continue;
          }

          if is_hidden {
            continue;
          }

          let is_intersected = self.window_list[this_wnd_idx]
            .popup
            .win
            .as_ref()
            .copied()
            .and_then(|popup_handle| {
              self.find_window_index_by_handle(popup_handle)
            })
            .map_or(false, |popup_win_idx| {
              self.window_list[popup_win_idx].bounds.intersect(&bounds)
            });

          if !is_intersected {
            continue;
          }

          //
          win_idx = this_wnd_idx;
          break;
        }
      }

      // let in_panel = in_panel
      //   && self
      //     .input
      //     .has_mouse_button_pressed(MouseButtonId::ButtonLeft);

      // activate window if clicked
      // if win_idx < self.window_list.len() && in_panel {}
    }

    false
  }

  pub fn panel_begin(&mut self, title: &str, panel_type: PanelType) -> bool {
    assert!(self.current.is_some());
    if !self.current.is_some() {
      return false;
    }

    // let current = self.current.as_ref().copied().unwrap();
    // let scrollbar_size = self.style.window.scrollbar_size;
    // let panel_padding = self.style.get_panel_padding(panel_type);

    // // window movement
    // if win.flags.contains(PanelFlags::WindowMovable)
    //   && !win.flags.contains(PanelFlags::WindowRom)
    // {}

    false
  }
}
