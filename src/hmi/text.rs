use enumflags2_derive::EnumFlags;
use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Copy, Clone, Debug, EnumFlags)]
pub enum TextAlign {
  AlignLeft = 0x01,
  AlignCentered = 0x02,
  AlignRight = 0x04,
  AlignTop = 0x08,
  AlignMiddle = 0x10,
  AlignBottom = 0x20,
}

#[derive(Copy, Clone, Debug, EnumFlags)]
pub enum TextAlignment {
  Left = 0x11,     // TextAlign::AlignMiddle | TextAlign::AlignLeft
  Centered = 0x12, // TextAlign::AlignMiddle | TextAlign::AlignCentered
  Right = 0x14,    // TextAlign::AlignMiddle | TextAlign::AlignRight
}
