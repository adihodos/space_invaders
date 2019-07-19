use crate::{
  hmi::commands::CommandBuffer,
  math::{rectangle::RectangleF32, vec2::Vec2U32},
};

use enumflags2_derive::EnumFlags;

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

pub struct PanelSet {}
impl PanelSet {
  // PanelType::Contextual  |  PanelType::Combo | PanelType::Menu |
  // PanelType::Tooltip
  pub const NON_BLOCK: u8 = 240;
  // PanelType::Contextual  |  PanelType::Combo |  PanelType::Menu |
  // PanelType::Tooltip | PanelType::Popup
  pub const POPUP: u8 = 244;
  // PanelType::Contextual  |  PanelType::Combo |
  // PanelType::Menu |  PanelType::Tooltip | PanelType::Popup
  // | PanelType::Group
  pub const SUB: u8 = 246;
}

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
pub struct PopupBuffer {
  pub begin:  u64,
  pub parent: u64,
  pub last:   u64,
  pub end:    u64,
  pub active: i32,
}

#[derive(Copy, Clone, Debug)]
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
  pub typ:           PanelType,
  pub flags:         u32,
  pub bounds:        RectangleF32,
  pub offset_x:      *mut u32,
  pub offset_y:      *mut u32,
  pub at_x:          f32,
  pub at_y:          f32,
  pub max_x:         f32,
  pub footer_height: f32,
  pub header_height: f32,
  pub border:        f32,
  pub has_scrolling: u32,
  pub clip:          RectangleF32,
  pub menu:          MenuState,
  pub row:           RowLayout,
  pub chart:         Chart,
  pub buffer:        *mut CommandBuffer,
  pub parent:        *mut Panel,
}
