use crate::math::vec2::Vec2F32;
use enumflags2::BitFlags;
use enumflags2_derive::EnumFlags;

pub type HashType = u64;

#[derive(Copy, Debug, Clone, PartialEq)]
pub enum GenericHandle {
  Ptr(usize),
  Id(u32),
}

#[derive(Copy, Debug, Clone)]
pub struct DrawNullTexture {
  /// <! texture handle to a texture containing a white pixel
  pub texture: GenericHandle,
  /// <! Coordinates of the white pixel in the above texture
  pub uv: Vec2F32,
}

impl std::default::Default for DrawNullTexture {
  fn default() -> DrawNullTexture {
    DrawNullTexture {
      texture: GenericHandle::Id(0),
      uv:      Vec2F32::new(0f32, 0f32),
    }
  }
}

#[derive(Debug, Clone)]
pub struct ConvertConfig {
  pub global_alpha:         f32,
  pub line_aa:              AntialiasingType,
  pub shape_aa:             AntialiasingType,
  pub circle_segment_count: u32,
  pub arc_segment_count:    u32,
  pub curve_segment_count:  u32,
  pub null:                 DrawNullTexture,
  pub vertex_layout:        Vec<DrawVertexLayoutElement>,
  pub vertex_size:          usize,
}

#[derive(Copy, Clone, Debug)]
pub enum ButtonBehaviour {
  ButtonDefault,
  ButtonRepeater,
}

impl std::default::Default for ButtonBehaviour {
  fn default() -> Self {
    ButtonBehaviour::ButtonDefault
  }
}

#[derive(Copy, Debug, Clone)]
pub struct UserFont {}

#[derive(Copy, Debug, Clone)]
pub struct PlaceholderType {}

#[derive(Copy, Debug, Clone, Eq, PartialEq)]
pub enum AntialiasingType {
  Off,
  On,
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum DrawVertexLayoutAttribute {
  Position,
  Color,
  Texcoord,
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum DrawVertexLayoutFormat {
  Schar,
  Sshort,
  Sint,
  Uchar,
  Ushort,
  Uint,
  Float,
  Double,
  FormatColorBegin,
  R8G8B8,
  R16G15B16,
  R32G32B32,
  R8G8B8A8,
  B8G8R8A8,
  R16G15B16A16,
  R32G32B32A32,
  R32G32B32A32_Float,
  R32G32B32A32_Double,
  RGB32,
  RGBA32,
  FormatColorEnd,
  FormatCount,
}

#[derive(Copy, Debug, Clone)]
pub struct DrawVertexLayoutElement {
  pub attribute: DrawVertexLayoutAttribute,
  pub format:    DrawVertexLayoutFormat,
  pub offset:    usize,
}

pub struct Consts {}

impl Consts {
  pub fn null_rect() -> crate::math::rectangle::RectangleF32 {
    crate::math::rectangle::RectangleF32::new(
      -8192_f32, -8192_f32, 16834_f32, 16834_f32,
    )
  }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum WidgetLayoutStates {
  Invalid,
  Valid,
  Rom,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, EnumFlags)]
pub enum WidgetStates {
  Modified = 1 << 1,
  Inactive = 1 << 2,
  Entered = 1 << 3,
  Hover = 1 << 4,
  Activated = 1 << 5,
  Left = 1 << 6,
}

impl WidgetStates {
  pub fn is_hovered(s: BitFlags<WidgetStates>) -> bool {
    s.contains(WidgetStates::Hover | WidgetStates::Modified)
  }

  pub fn hovered() -> BitFlags<WidgetStates> {
    WidgetStates::Hover | WidgetStates::Modified
  }

  pub fn is_active(s: BitFlags<WidgetStates>) -> bool {
    s.contains(WidgetStates::Activated | WidgetStates::Modified)
  }

  pub fn active() -> BitFlags<WidgetStates> {
    WidgetStates::Activated | WidgetStates::Modified
  }

  pub fn reset(s: BitFlags<WidgetStates>) -> BitFlags<WidgetStates> {
    if s.contains(WidgetStates::Modified) {
      WidgetStates::Inactive | WidgetStates::Modified
    } else {
      WidgetStates::Inactive | WidgetStates::Inactive
    }
  }
}
