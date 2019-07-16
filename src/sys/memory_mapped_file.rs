#![allow(dead_code)]

use crate::sys::unique_resource::UniqueResource;
use std::path::Path;

#[cfg(unix)]
mod unix {
  use crate::sys::unique_resource::ResourceDeleter;
  use libc::{c_int, c_void, close, munmap};

  #[derive(Default)]
  pub struct OSFileDeleter {}

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

  pub struct MemoryMappingDeleter {
    size: usize,
  }

  impl MemoryMappingDeleter {
    pub fn new(size: usize) -> Self {
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
} // mod unix

#[cfg(windows)]
mod win32 {
  use std::{
    os::windows::prelude::*,
    ptr::{null, null_mut},
  };

  use winapi::{
    shared::minwindef::LPVOID,
    um::{
      handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
      memoryapi::UnmapViewOfFile,
      winnt::HANDLE,
    },
  };

  use crate::sys::unique_resource::ResourceDeleter;

  pub fn win_str(s: &str) -> Vec<u16> {
    std::ffi::OsStr::new(s)
      .encode_wide()
      .chain(std::iter::once(0))
      .collect()
  }

  pub fn path_to_win_str<P: AsRef<std::path::Path>>(p: P) -> Vec<u16> {
    p.as_ref()
      .as_os_str()
      .encode_wide()
      .chain(std::iter::once(0))
      .collect()
  }

  #[derive(Default)]
  pub struct MemoryMappingDeleter {}

  impl ResourceDeleter for MemoryMappingDeleter {
    type Handle = LPVOID;

    fn is_null(res: &Self::Handle) -> bool {
      *res == Self::null()
    }

    fn null() -> Self::Handle {
      std::ptr::null_mut()
    }

    fn delete(&mut self, res: &mut Self::Handle) {
      unsafe {
        UnmapViewOfFile(*res);
      }
    }
  }

  #[derive(Default)]
  pub struct OSFileDeleter {}

  impl ResourceDeleter for OSFileDeleter {
    type Handle = HANDLE;

    fn is_null(res: &Self::Handle) -> bool {
      *res == Self::null()
    }

    fn null() -> Self::Handle {
      INVALID_HANDLE_VALUE
    }

    fn delete(&mut self, res: &mut Self::Handle) {
      unsafe {
        CloseHandle(*res);
      }
    }
  }

  #[derive(Default)]
  pub struct FileMappingDeleter {}

  impl ResourceDeleter for FileMappingDeleter {
    type Handle = HANDLE;

    fn is_null(res: &Self::Handle) -> bool {
      *res == Self::null()
    }

    fn null() -> Self::Handle {
      null_mut()
    }

    fn delete(&mut self, res: &mut Self::Handle) {
      unsafe {
        CloseHandle(*res);
      }
    }
  }
} // mod win32

#[cfg(unix)]
type MemoryMappingDeleter =
  crate::sys::memory_mapped_file::unix::MemoryMappingDeleter;
#[cfg(unix)]
type OSFileDeleter = crate::sys::memory_mapped_file::unix::OSFileDeleter;
#[cfg(windows)]
type MemoryMappingDeleter =
  crate::sys::memory_mapped_file::win32::MemoryMappingDeleter;
#[cfg(windows)]
type OSFileDeleter = crate::sys::memory_mapped_file::win32::OSFileDeleter;
#[cfg(windows)]
type FileMappingDeleter =
  crate::sys::memory_mapped_file::win32::FileMappingDeleter;

/// A file mapped into the memory of the process. Contents can be accessed as
/// a byte slice. Read-only access.
pub struct MemoryMappedFile {
  /// starting address where file was mapped in memory
  memory: UniqueResource<MemoryMappingDeleter>,
  /// handle to the file
  file_handle: UniqueResource<OSFileDeleter>,
  /// length in bytes of the mapping
  bytes: usize,
}

impl MemoryMappedFile {
  // Construct by mapping the specified file into memory
  #[cfg(unix)]
  pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<MemoryMappedFile> {
    use libc::{mmap, open, MAP_PRIVATE, O_RDONLY, PROT_READ};
    use std::{
      ffi::CString,
      io::{Error, ErrorKind},
      ptr::null_mut,
    };

    let metadata = std::fs::metadata(&path)?;

    path
      .as_ref()
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
                  memory:      ummap,
                  file_handle: ufd,
                  bytes:       metadata.len() as usize,
                })
              })
            })
          })
      })
  }

  /// Construct by mapping the specified file into memory
  #[cfg(windows)]
  pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<MemoryMappedFile> {
    use std::{
      io::{Error, ErrorKind},
      ptr::null_mut,
    };
    use winapi::um::{
      fileapi::{CreateFileW, OPEN_EXISTING},
      memoryapi::{CreateFileMappingW, MapViewOfFile, FILE_MAP_READ},
      winnt::{
        FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, GENERIC_READ, GENERIC_WRITE,
        PAGE_READONLY,
      },
    };

    let win_path =
      crate::sys::memory_mapped_file::win32::path_to_win_str(&path);
    let metadata = std::fs::metadata(&path)?;

    UniqueResource::<OSFileDeleter>::from_handle(unsafe {
      CreateFileW(
        win_path.as_ptr(),
        GENERIC_READ,
        FILE_SHARE_READ,
        null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        null_mut(),
      )
    })
    .ok_or(Error::last_os_error())
    .and_then(|file_handle| {
      // Use the file handle to create a file mapping object. This gets
      // destroyed once we leave the closure, but it is not  a problem
      // since the mapping of the file stays valid untill all mapped views
      // are closed.
      UniqueResource::<FileMappingDeleter>::from_handle(unsafe {
        CreateFileMappingW(
          *file_handle.handle(),
          null_mut(),
          PAGE_READONLY,
          0,
          0,
          null_mut(),
        )
      })
      .ok_or(Error::last_os_error())
      .and_then(|file_mapping| {
        // finally we can map the view of file in the process memory
        UniqueResource::<MemoryMappingDeleter>::from_handle(unsafe {
          MapViewOfFile(*file_mapping.handle(), FILE_MAP_READ, 0, 0, 0)
        })
        .ok_or(Error::last_os_error())
        .and_then(|memory| {
          Ok(MemoryMappedFile {
            memory,
            file_handle,
            bytes: metadata.len() as usize,
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
      std::slice::from_raw_parts(*self.memory.handle() as *const u8, self.bytes)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use std::{ffi::CStr, fs::File, io::prelude::*, os::raw::c_char, path::Path};

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
