use crate::{
  hmi::{
    base::{AntialiasingType, ButtonBehaviour, ConvertConfig, GenericHandle},
    commands::CommandBuffer,
    input::Input,
    panel::Panel,
    style::{ConfigurationStacks, Style},
    text_engine::Font,
    vertex_output::{DrawCommand, DrawIndexType, DrawList},
    window::{Window, WindowFlags},
  },
  math::{rectangle::RectangleF32, vertex_types::VertexPTC},
};

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
  build:       i32,
  window_list: Vec<Window>,
  active:      Option<usize>,
  current:     *mut Window,
  count:       u32,
  seq:         u32,
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
      current:           std::ptr::null_mut(),
      count:             0,
      seq:               0,
    }
  }

  fn is_active_window(&self, idx: usize) -> bool {
    self.active.map_or(false, |active_idx| active_idx == idx)
  }


  pub fn clear(&mut self) {
    self.build = 0;
    self.last_widget_state = 0;
    self.style.cursor_active = 0; // FIX this
    self.overlay.clear();

//    let to_remove = self.window_list.iter().filter(|win| {
//      if (win.flags & WindowFlags::MINIMIZED) != 0 && (win.flags & WindowFlags::CLOSED) == 0 && win.seq == self.seq {
//        false
//      }else {
//        true
//      }
//    }).collect::<Vec<_>>();

    let mut wnd_list = std::mem::replace(&mut self.window_list, vec![]);

    for (idx, win) in wnd_list.iter_mut().enumerate() {
      // make sure valid windows do not get removed
      if (win.flags & WindowFlags::MINIMIZED) != 0 && (win.flags & WindowFlags::CLOSED) == 0 && win.seq == self.seq {
        continue;
      }

      // remove hotness from hidden or closed windows
      if (win.flags & WindowFlags::HIDDEN) != 0 && (win.flags & WindowFlags::CLOSED) != 0 &&  self.is_active_window( idx) {
        self.active = if idx > 0 { Some(idx - 1) } else {None};
        self.active.as_ref().copied().and_then(|id| {
          self.window_list[id].flags &= !WindowFlags::ROM;
          Some(())
        });

        // free unused popup windows
        if !win.popup.win.is_null() && win.popup.win.seq != self.seq {

        }

      }

    }


  }
}
