//use enumflags2_derive::EnumFlags;
//use num_derive::{FromPrimitive, ToPrimitive};

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

//#[derive(Copy, Clone, Debug, EnumFlags)]
//pub enum TextAlignment {
//  Left = 0x11,     // TextAlign::AlignMiddle | TextAlign::AlignLeft
//  Centered = 0x12, // TextAlign::AlignMiddle | TextAlign::AlignCentered
//  Right = 0x14,    // TextAlign::AlignMiddle | TextAlign::AlignRight
//}

pub struct TextAlignment {}

impl TextAlignment {
  pub const Left : u32 =     TextAlign::AlignMiddle as u32 | TextAlign::AlignLeft as u32;
  pub const Centered : u32 =  TextAlign::AlignMiddle as u32 | TextAlign::AlignCentered as u32;
  pub const Right : u32 =  TextAlign::AlignMiddle as u32 | TextAlign::AlignRight as u32;

}