#![allow(dead_code)]

pub trait ResourceDeleter {
  type Handle;

  fn is_null(res: &Self::Handle) -> bool;
  fn null() -> Self::Handle;
  fn delete(&mut self, res: &mut Self::Handle);
}

pub struct UniqueResource<T: ResourceDeleter> {
  handle:  <T as ResourceDeleter>::Handle,
  deleter: T,
}

impl<T: ResourceDeleter> UniqueResource<T> {
  pub fn from_handle(handle: <T as ResourceDeleter>::Handle) -> Option<Self>
  where
    T: std::default::Default,
  {
    Self::from_state_handle(handle, T::default())
  }

  pub fn from_state_handle(
    handle: <T as ResourceDeleter>::Handle,
    deleter: T,
  ) -> Option<Self> {
    if T::is_null(&handle) {
      None
    } else {
      Some(Self { handle, deleter })
    }
  }

  pub fn handle(&self) -> &<T as ResourceDeleter>::Handle {
    &self.handle
  }
}

impl<T: ResourceDeleter> std::ops::Drop for UniqueResource<T> {
  fn drop(&mut self) {
    if !T::is_null(&self.handle) {
      self.deleter.delete(&mut self.handle)
    }
  }
}
