
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum TextAlign {
  AlignLeft = 0x01,
  AlignCentered = 0x02,
  AlignRight = 0x04,
  AlignTop = 0x08,
  AlignMiddle = 0x10,
  AlignBottom = 0x20,
}

pub struct TextAlignment {}

impl TextAlignment {
  pub const CENTERED: u32 =
    TextAlign::AlignMiddle as u32 | TextAlign::AlignCentered as u32;
  pub const LEFT: u32 =
    TextAlign::AlignMiddle as u32 | TextAlign::AlignLeft as u32;
  pub const RIGHT: u32 =
    TextAlign::AlignMiddle as u32 | TextAlign::AlignRight as u32;
}
