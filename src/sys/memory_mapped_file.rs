#![allow(dead_code)]

#[cfg(unix)]
mod unix {
  use crate::sys::unique_resource::{ResourceDeleter, UniqueResource};

  use libc::{
    c_int, c_void, close, mmap, munmap, open, MAP_PRIVATE, O_RDONLY, PROT_READ,
  };
  use std::{ffi::CString, path::Path, ptr::null_mut};

  #[derive(Default)]
  struct OSFileDeleter {}

  impl ResourceDeleter for OSFileDeleter {
    type Handle = c_int;

    fn is_null(res: &Self::Handle) -> bool {
      *res == -1
    }

    fn null() -> Self::Handle {
      -1
    }

    fn delete(&mut self, res: &mut Self::Handle) {
      unsafe {
        close(*res);
      }
    }
  }

  struct MemoryMappingDeleter {
    size: usize,
  }

  impl MemoryMappingDeleter {
    fn new(size: usize) -> Self {
      Self { size }
    }
  }

  impl ResourceDeleter for MemoryMappingDeleter {
    type Handle = *mut c_void;

    fn is_null(res: &Self::Handle) -> bool {
      *res == std::ptr::null_mut()
    }

    fn null() -> Self::Handle {
      std::ptr::null_mut()
    }

    fn delete(&mut self, res: &mut Self::Handle) {
      if !Self::is_null(res) {
        unsafe {
          munmap(*res, self.size);
        }
      }
    }
  }

  /// A file mapped into the memory of the process. Contents can be accessed as
  /// a byte slice.
  pub struct MemoryMappedFile {
    memory: UniqueResource<MemoryMappingDeleter>,
    bytes: usize,
    file_handle: UniqueResource<OSFileDeleter>,
  }

  impl MemoryMappedFile {
    // Construct by mapping the specified file into memory
    pub fn new(path: &Path) -> std::io::Result<MemoryMappedFile> {
      let metadata = std::fs::metadata(path)?;

      use std::io::{Error, ErrorKind};

      path
        .to_str()
        .ok_or(Error::new(ErrorKind::InvalidData, "plm"))
        .and_then(|str_path| {
          // convert path to C-string
          CString::new(str_path.as_bytes())
            .map_err(|_| Error::new(ErrorKind::InvalidData, "plm"))
            .and_then(|cstr_path| {
              // open file
              UniqueResource::<OSFileDeleter>::from_handle(unsafe {
                open(cstr_path.as_c_str().as_ptr(), O_RDONLY)
              })
              .ok_or(Error::last_os_error())
              .and_then(|ufd| {
                // map into memory
                UniqueResource::<MemoryMappingDeleter>::from_state_handle(
                  unsafe {
                    mmap(
                      null_mut(),
                      metadata.len() as usize,
                      PROT_READ,
                      MAP_PRIVATE,
                      *ufd.handle(),
                      0,
                    )
                  },
                  MemoryMappingDeleter::new(metadata.len() as usize),
                )
                .ok_or(Error::last_os_error())
                .and_then(|ummap| {
                  Ok(MemoryMappedFile {
                    memory: ummap,
                    bytes: metadata.len() as usize,
                    file_handle: ufd,
                  })
                })
              })
            })
        })
    }

    /// Returns the length in bytes of the file that was mapped in memory.
    pub fn len(&self) -> usize {
      self.bytes
    }

    /// Returns a slice spanning the contents of the file that was mapped in
    /// memory
    pub fn as_slice(&self) -> &[u8] {
      unsafe {
        std::slice::from_raw_parts(
          *self.memory.handle() as *const u8,
          self.bytes,
        )
      }
    }
  }
}

#[cfg(unix)]
pub use self::unix::MemoryMappedFile;

#[cfg(test)]
mod tests {
  use super::*;

  use std::{ffi::CStr, os::raw::c_char};
  use std::{fs::File, io::prelude::*, path::Path};

  #[test]
  fn test_memory_mapped_file() {
    let mmfile = MemoryMappedFile::new(Path::new("non-existing-test-file.txt"));
    assert!(mmfile.is_err());

    let txt = b"A memory mapped file\0";
    {
      let mut f = File::create("test.txt").unwrap();
      f.write_all(txt).unwrap();
    }

    let mmfile = MemoryMappedFile::new(Path::new("test.txt"));
    assert!(!mmfile.is_err());
    let mmfile = mmfile.unwrap();

    unsafe {
      let m = CStr::from_ptr(mmfile.as_slice().as_ptr() as *const c_char);
      let org = CStr::from_bytes_with_nul(b"A memory mapped file\0").unwrap();
      assert_eq!(m, org);
    }
  }
}
