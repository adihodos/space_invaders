use crate::{
  hmi::{
    commands::CommandBuffer,
    panel::{Panel, PanelType, PopupBuffer},
  },
  math::{rectangle::RectangleF32, vec2::Vec2U32},
};

pub struct WindowFlags {}

impl WindowFlags {
  pub const CLOSED: u32 = 1 << 14;
  pub const DYNAMIC: u32 = WindowFlags::PRIVATE;
  pub const HIDDEN: u32 = 1 << 13;
  pub const MINIMIZED: u32 = 1 << 15;
  pub const NOT_INTERACTIVE: u32 = WindowFlags::ROM | 1 << 10;
  pub const PRIVATE: u32 = 1 << 11;
  pub const REMOVE_ROM: u32 = 1 << 16;
  pub const ROM: u32 = 1 << 12;
}

#[derive(Copy, Clone, Debug)]
pub struct PopupState {
  pub win:         *mut Window,
  pub typ:         PanelType,
  pub buf:         PopupBuffer,
  pub name:        u32,
  pub active:      bool,
  pub combo_count: u32,
  pub con_count:   u32,
  pub con_old:     u32,
  pub active_con:  u32,
  pub header:      RectangleF32,
}

#[derive(Copy, Clone, Debug)]
pub struct EditState {
  pub name:        u32,
  pub seq:         u32,
  pub old:         u32,
  pub active:      i32,
  pub prev:        i32,
  pub cursor:      i32,
  pub sel_start:   i32,
  pub sel_end:     i32,
  pub scrollbar:   Vec2U32,
  pub mode:        u8,
  pub single_line: u8,
}

#[derive(Clone, Debug)]
pub struct PropertyState {
  pub active:       i32,
  pub prev:         i32,
  pub buffer:       String,
  pub length:       i32,
  pub cursor:       i32,
  pub select_start: i32,
  pub select_end:   i32,
  pub name:         u32,
  pub seq:          u32,
  pub old:          u32,
  pub state:        i32,
}

#[derive(Debug)]
pub struct Window {
  pub seq:                    u32,
  pub name:                   u32,
  pub name_str:               String,
  pub flags:                  u32,
  pub bounds:                 RectangleF32,
  pub scrollbar:              Vec2U32,
  pub buffer:                 CommandBuffer,
  pub layout:                 *mut Panel,
  pub scrollbar_hiding_timer: f32,
  // persistent widget state
  pub property: PropertyState,
  pub popup:    PopupState,
  pub edit:     EditState,
  pub scrolled: u32,

  // tables ??!!

  // window list hooks
  pub prev:   *mut Window,
  pub next:   *mut Window,
  pub parent: *mut Window,
}
