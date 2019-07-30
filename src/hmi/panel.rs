use crate::{
  hmi::{base::Consts, commands::CommandBuffer, style::Style, window::Window},
  math::{rectangle::RectangleF32, vec2::Vec2U32},
};

use enumflags2::BitFlags;
use enumflags2_derive::EnumFlags;
use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LayoutFormat {
  Dynamic,
  Static,
}

pub const MAX_LAYOUT_ROW_TEMPLATE_COLUMNS: usize = 16;
pub const MAX_CHART_SLOT: usize = 4;

#[derive(EnumFlags, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum PanelType {
  Window = 1u8 << 0,
  Group = 1u8 << 1,
  Popup = 1u8 << 2,
  Contextual = 1u8 << 4,
  Combo = 1u8 << 5,
  Menu = 1u8 << 6,
  Tooltip = 1u8 << 7,
}

impl PanelType {
  // pub fn is_nonblock(&self) -> bool {
  //   *self as u8 & PanelSet::NON_BLOCK != 0
  // }
  fn non_block() -> BitFlags<PanelType> {
    PanelType::Contextual
      | PanelType::Combo
      | PanelType::Menu
      | PanelType::Tooltip
  }

  fn popup() -> BitFlags<PanelType> {
    PanelType::Contextual
      | PanelType::Combo
      | PanelType::Menu
      | PanelType::Tooltip
      | PanelType::Popup
  }

  fn sub() -> BitFlags<PanelType> {
    PanelType::Contextual
      | PanelType::Combo
      | PanelType::Menu
      | PanelType::Tooltip
      | PanelType::Popup
      | PanelType::Group
  }
}

#[derive(
  EnumFlags, Copy, Clone, Debug, PartialEq, FromPrimitive, ToPrimitive,
)]
#[repr(u32)]
pub enum PanelFlags {
  WindowBorder = 1 << 0,
  WindowMovable = 1 << 1,
  WindowScalable = 1 << 2,
  WindowClosable = 1 << 3,
  WindowMinimizable = 1 << 4,
  WindowNoScrollbar = 1 << 5,
  WindowTitle = 1 << 6,
  WindowScrollAutoHide = 1 << 7,
  WindowBackground = 1 << 8,
  WindowScaleLeft = 1 << 9,
  WindowNoInput = 1 << 10,
  WindowDynamic = 1 << 11,
  WindowRom = 1 << 12,
  WindowHidden = 1 << 13,
  WindowClosed = 1 << 14,
  WindowMinimized = 1 << 15,
  WindowRemoveRom = 1 << 16,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PanelRowLayoutType {
  DynamicFixed,
  DynamicRow,
  DynamicFree,
  Dynamic,
  StaticFixed,
  StaticRow,
  StaticFree,
  Static,
  Template,
  Count,
}

impl std::default::Default for PanelRowLayoutType {
  fn default() -> PanelRowLayoutType {
    PanelRowLayoutType::Count
  }
}

#[derive(Copy, Clone, Debug)]
pub struct RowLayout {
  pub typ:         PanelRowLayoutType,
  pub index:       i32,
  pub height:      f32,
  pub min_height:  f32,
  pub columns:     i32,
  pub ratio:       *const f32,
  pub item_width:  f32,
  pub item_height: f32,
  pub item_offset: f32,
  pub filled:      f32,
  pub item:        RectangleF32,
  pub tree_depth:  i32,
  pub templates:   [f32; MAX_LAYOUT_ROW_TEMPLATE_COLUMNS],
}

impl std::default::Default for RowLayout {
  fn default() -> RowLayout {
    RowLayout {
      typ:         PanelRowLayoutType::default(),
      index:       0,
      height:      0f32,
      min_height:  0f32,
      columns:     0,
      ratio:       std::ptr::null_mut(),
      item_width:  0f32,
      item_height: 0f32,
      item_offset: 0f32,
      filled:      0f32,
      item:        RectangleF32::new(0f32, 0f32, 0f32, 0f32),
      tree_depth:  0,
      templates:   [0f32; MAX_LAYOUT_ROW_TEMPLATE_COLUMNS],
    }
  }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct PopupBuffer {
  pub begin:  u64,
  pub parent: u64,
  pub last:   u64,
  pub end:    u64,
  pub active: i32,
}

// impl std::default::Default for PopupBuffer {
//   fn default() -> Self {
//     Self {
//       begin:  0,
//       parent: 0,
//       last:   0,
//       end:    0,
//       active: 0,
//     }
//   }
// }

#[derive(Copy, Clone, Debug, Default)]
pub struct MenuState {
  pub x:      f32,
  pub y:      f32,
  pub w:      f32,
  pub h:      f32,
  pub offset: Vec2U32,
}

#[derive(Copy, Clone, Debug)]
pub struct Chart {}

#[derive(Copy, Clone, Debug)]
pub struct Panel {
  pub typ:           BitFlags<PanelType>,
  pub flags:         BitFlags<PanelFlags>,
  pub bounds:        RectangleF32,
  pub offset_x:      *mut u32,
  pub offset_y:      *mut u32,
  pub at_x:          f32,
  pub at_y:          f32,
  pub max_x:         f32,
  pub footer_height: f32,
  pub header_height: f32,
  pub border:        f32,
  pub has_scrolling: bool,
  pub clip:          RectangleF32,
  pub menu:          MenuState,
  pub row:           RowLayout,
  pub chart:         Chart,
  pub buffer:        *mut CommandBuffer,
  pub parent:        *mut Panel,
}

impl Panel {
  pub fn new(typ: BitFlags<PanelType>) -> Panel {
    Panel {
      typ,
      flags: BitFlags::<PanelFlags>::empty(),
      bounds: RectangleF32::new(0f32, 0f32, 0f32, 0f32),
      offset_x: std::ptr::null_mut(),
      offset_y: std::ptr::null_mut(),
      at_x: 0f32,
      at_y: 0f32,
      max_x: 0f32,
      footer_height: 0f32,
      header_height: 0f32,
      border: 0f32,
      has_scrolling: false,
      clip: Consts::null_rect(),
      menu: MenuState::default(),
      row: RowLayout::default(),
      chart: Chart {},
      buffer: std::ptr::null_mut(),
      parent: std::ptr::null_mut(),
    }
  }

  pub fn has_header(flags: BitFlags<PanelFlags>, title: Option<&str>) -> bool {
    let active = flags
      .contains(PanelFlags::WindowClosable | PanelFlags::WindowMinimizable);
    let active = active || flags.contains(PanelFlags::WindowTitle);
    let active =
      active && !flags.contains(PanelFlags::WindowHidden) && title.is_some();
    active
  }

  pub fn is_nonblock(&self) -> bool {
    self.typ.contains(PanelType::non_block())
  }

  pub fn is_popup(&self) -> bool {
    self.typ.contains(PanelType::popup())
  }

  pub fn is_sub(&self) -> bool {
    self.typ.contains(PanelType::sub())
  }

  pub fn reset_min_row_height(&mut self, style: &Style) {
    self.row.min_height = style.font.scale;
    self.row.min_height += style.text.padding.y * 2f32;
    self.row.min_height += style.window.min_row_height_padding * 2f32;
  }

  pub fn set_min_row_height(&mut self, height: f32) {
    self.row.min_height = height;
  }
}
