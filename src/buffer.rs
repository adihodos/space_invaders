pub trait Buffer<E> {
  type Element;

  fn len(&self) -> usize;
  fn alloc_elements(&mut self, count: usize) -> &mut [Self::Element];
  fn clear(&mut self);
  fn elements(&self) -> &[Self::Element];
  fn elements_mut(&mut self) -> &mut [Self::Element];
  fn push(&mut self, e: Self::Element);
  fn element_size() -> usize {
    std::mem::size_of::<E>()
  }
  fn element_alignment() -> usize {
    std::mem::align_of::<E>()
  }
}

pub struct DynamicBuffer<T>
where
  T: Copy + Clone,
{
  mem: Vec<T>,
}

impl<T> DynamicBuffer<T>
where
  T: Copy + Clone + std::default::Default,
{
  pub fn new() -> Self {
    Self::new_with_capacity(64)
  }

  pub fn new_with_capacity(cap: usize) -> Self {
    DynamicBuffer {
      mem: Vec::with_capacity(cap),
    }
  }
}

impl<T> Buffer<T> for DynamicBuffer<T>
where
  T: Copy + Clone + std::default::Default,
{
  type Element = T;

  fn len(&self) -> usize {
    self.mem.len()
  }

  fn alloc_elements(&mut self, count: usize) -> &mut [Self::Element] {
    let start_idx = self.mem.len();
    self.mem.resize_with(start_idx + count, || T::default());
    &mut self.mem[start_idx..]
  }

  fn clear(&mut self) {
    self.mem.clear()
  }

  fn elements(&self) -> &[Self::Element] {
    &self.mem
  }

  fn elements_mut(&mut self) -> &mut [Self::Element] {
    &mut self.mem
  }

  fn push(&mut self, e: Self::Element) {
    self.mem.push(e)
  }
}

pub struct FixedBuffer<T>
where
  T: Copy + Clone,
{
  ptr: *mut T,
  cap: usize,
  len: usize,
}

impl<T> FixedBuffer<T>
where
  T: Copy + Clone,
{
  pub fn new(ptr: *mut T, element_count: usize) -> Self {
    FixedBuffer {
      ptr,
      cap: element_count,
      len: 0,
    }
  }

  pub fn from_slice(s: &mut [T]) -> Self {
    Self::new(s.as_mut_ptr(), s.len())
  }
}

impl<T> Buffer<T> for FixedBuffer<T>
where
  T: Copy + Clone,
{
  type Element = T;

  fn len(&self) -> usize {
    self.len
  }

  fn alloc_elements(&mut self, count: usize) -> &mut [Self::Element] {
    let remaining = self.cap - self.len;
    if remaining < count {
      panic!("Fixed buffer capacity exceeded!");
    }

    let start_idx = self.len;
    self.len += count;
    unsafe {
      std::slice::from_raw_parts_mut(self.ptr.offset(start_idx as isize), self.len - start_idx)
    }
  }

  fn clear(&mut self) {
    self.len = 0;
  }

  fn elements(&self) -> &[Self::Element] {
    unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
  }

  fn elements_mut(&mut self) -> &mut [Self::Element] {
    unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
  }

  fn push(&mut self, e: Self::Element) {
    if self.len >= self.cap {
      panic!("push() => fixed buffer capacity exceeded");
    }

    unsafe {
      self.ptr.offset(self.len as isize).write(e);
      self.len += 1;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_dyn_buffer_alloc() {
    let mut dyb = DynamicBuffer::<i32>::new();

    {
      let mut new_slice = dyb.alloc_elements(5);
      (0..5).for_each(|idx| {
        let src = idx as i32 * 4;
        unsafe {
          std::ptr::copy_nonoverlapping(
            &src as *const i32,
            new_slice.as_mut_ptr().offset(idx as isize),
            1,
          );
        }
      });

      assert_eq!(new_slice[0], 0);
      assert_eq!(new_slice[1], 4);
      assert_eq!(new_slice[2], 8);
      assert_eq!(new_slice[3], 12);
      assert_eq!(new_slice[4], 16);
    }

    dyb.push(20);

    let elements = dyb.elements();
    assert_eq!(elements[0], 0);
    assert_eq!(elements[1], 4);
    assert_eq!(elements[2], 8);
    assert_eq!(elements[3], 12);
    assert_eq!(elements[4], 16);
    assert_eq!(elements[5], 20);
  }

  #[test]
  fn test_fixed_buffer() {
    let mut fbstore: [i32; 16] = [0; 16];

    let mut fixb = FixedBuffer::<i32>::from_slice(&mut fbstore);

    fixb.push(1);
    fixb.push(2);
    fixb.push(3);
    fixb.push(4);

    assert_eq!(fixb.len(), 4);

    assert_eq!(fbstore[0], 1);
    assert_eq!(fbstore[1], 2);
    assert_eq!(fbstore[2], 3);
    assert_eq!(fbstore[3], 4);

    {
      let mut new_slice = fixb.alloc_elements(4);
      new_slice[0] = 5;
      new_slice[1] = 6;
      new_slice[2] = 7;
      new_slice[3] = 8;

      assert_eq!(fbstore[4], 5);
      assert_eq!(fbstore[5], 6);
      assert_eq!(fbstore[6], 7);
      assert_eq!(fbstore[7], 8);
    }

    let elements = fixb.elements();
    elements
      .iter()
      .enumerate()
      .for_each(|(i, v)| assert_eq!(i as i32 + 1, *v));
  }
}