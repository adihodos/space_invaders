use crate::{hmi::base::GenericHandle, math::rectangle::RectangleF32};

#[derive(Copy, Debug, Clone)]
pub struct Image {
  pub handle: GenericHandle,
  pub w:      u16,
  pub h:      u16,
  pub region: [u16; 4],
}

impl Image {
  pub fn subimage_ptr(ptr: usize, w: u16, h: u16, r: RectangleF32) -> Image {
    Self::subimage_handle(GenericHandle::Ptr(ptr), w, h, r)
  }

  pub fn subimage_id(id: u32, w: u16, h: u16, r: RectangleF32) -> Image {
    Self::subimage_handle(GenericHandle::Id(id), w, h, r)
  }

  pub fn subimage_handle(
    handle: GenericHandle,
    w: u16,
    h: u16,
    r: RectangleF32,
  ) -> Image {
    Image {
      handle,
      w,
      h,
      region: [r.x as u16, r.y as u16, r.w as u16, r.h as u16],
    }
  }

  pub fn image_handle(handle: GenericHandle) -> Image {
    Image {
      handle,
      w: 0,
      h: 0,
      region: [0u16; 4],
    }
  }

  pub fn image_ptr(ptr: usize) -> Image {
    Self::image_handle(GenericHandle::Ptr(ptr))
  }

  pub fn image_id(id: u32) -> Image {
    Self::image_handle(GenericHandle::Id(id))
  }

  pub fn is_subimage(&self) -> bool {
    self.w != 0 && self.h != 0
  }
}
