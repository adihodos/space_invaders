use crate::{
  hmi::{
    base::HashType,
    commands::CommandBuffer,
    panel::{Panel, PanelFlags, PanelType, PopupBuffer},
  },
  math::{rectangle::RectangleF32, vec2::Vec2U32},
};
use enumflags2::BitFlags;
use std::{cell::RefCell, rc::Rc};

#[derive(Copy, Clone, Debug)]
pub struct PopupState {
  pub win:         Option<usize>,
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

impl std::default::Default for PopupState {
  fn default() -> Self {
    Self {
      win:         None,
      typ:         PanelType::Popup,
      buf:         PopupBuffer::default(),
      name:        0,
      active:      false,
      combo_count: 0,
      con_count:   0,
      con_old:     0,
      active_con:  0,
      header:      RectangleF32::new(0f32, 0f32, 0f32, 0f32),
    }
  }
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

impl std::default::Default for EditState {
  fn default() -> Self {
    Self {
      name:        0,
      seq:         0,
      old:         0,
      active:      0,
      prev:        0,
      cursor:      0,
      sel_start:   0,
      sel_end:     0,
      scrollbar:   Vec2U32::same(0),
      mode:        0,
      single_line: 0,
    }
  }
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

impl std::default::Default for PropertyState {
  fn default() -> Self {
    Self {
      active:       0,
      prev:         0,
      buffer:       String::new(),
      length:       0,
      cursor:       0,
      select_start: 0,
      select_end:   0,
      name:         0,
      seq:          0,
      old:          0,
      state:        0,
    }
  }
}

#[derive(Debug)]
pub struct Window {
  pub handle:                 usize,
  pub seq:                    u32,
  pub name:                   HashType,
  pub name_str:               String,
  pub flags:                  BitFlags<PanelFlags>,
  pub bounds:                 RectangleF32,
  pub scrollbar:              Vec2U32,
  pub buffer:                 RefCell<CommandBuffer>,
  pub layout:                 Box<RefCell<Panel>>,
  pub scrollbar_hiding_timer: f32,
  // persistent widget state
  pub property: PropertyState,
  pub popup:    PopupState,
  pub edit:     EditState,
  pub scrolled: u32,

  // tables ??!!

  // window list hooks

  // pub prev:   *mut Window,
  // pub next:   *mut Window,
  // pub parent: *mut Window,
  pub parent: Option<Rc<RefCell<Window>>>,
}

impl Window {
  pub fn new(
    handle: usize,
    name: HashType,
    name_str: &str,
    flags: BitFlags<PanelFlags>,
    bounds: RectangleF32,
  ) -> Window {
    Window {
      handle,
      seq: 0,
      name,
      name_str: name_str.to_owned(),
      flags,
      bounds,
      scrollbar: Vec2U32::same(0),
      buffer: RefCell::new(CommandBuffer::new(
        Some(RectangleF32::new(
          -8192_f32, -8192_f32, 16834_f32, 16834_f32,
        )),
        128,
      )),
      layout: Box::new(RefCell::new(Panel::new(PanelType::Window.into()))),
      scrollbar_hiding_timer: 0f32,
      property: PropertyState::default(),
      popup: PopupState::default(),
      edit: EditState::default(),
      scrolled: 0,
      parent: None,
    }
  }
}
