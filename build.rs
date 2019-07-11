// extern crate cmake;

use std::{
  env,
  ffi::OsString,
  fs::{self, File},
  io::prelude::*,
  path::{Path, PathBuf},
  process::Command,
};

macro_rules! t {
  ($e:expr) => {
    match $e {
      Ok(e) => e,
      Err(e) => panic!("{} failed with {}", stringify!($e), e),
    }
  };
}

fn dump_vars() {
  for (var_name, var_val) in std::env::vars() {
    println!("{} -> {}", var_name, var_val);
  }
}

fn main() {
  dump_vars();
  let _ = fs::remove_dir_all(env::var("OUT_DIR").unwrap());
  fs::create_dir_all(env::var("OUT_DIR").unwrap());

  env::remove_var("DESTDIR");

  let target = env::var("TARGET").unwrap();
  let host = env::var("HOST").unwrap();
  let windows = target.contains("windows");
  let msvc = target.contains("msvc");

  let mut cfg = cmake::Config::new("src/third_party/freetype2");

  cfg
    .define("DISABLE_FORCE_DEBUG_POSTFIX", "ON")
    .define("BUILD_SHARED_LIBS", "OFF")
    .define("CMAKE_DISABLE_FIND_PACKAGE_HarfBuzz", "TRUE")
    .define("CMAKE_DISABLE_FIND_PACKAGE_PNG", "TRUE")
    .define("CMAKE_DISABLE_FIND_PACKAGE_ZLIB", "TRUE")
    .define("CMAKE_DISABLE_FIND_PACKAGE_BZip2", "TRUE");

  // if !msvc {
  //   cfg.cflag("-fPIC");
  // }

  let dst = cfg.build();

  ["lib", "lib64"].iter().any(|path| {
    let p = dst.clone().join(path);
    if p.exists() {
      println!("cargo:rustc-link-search=dependency={}", p.display());
      println!("cargo:rustc-link-lib=static=freetype");
      return true;
    }

    false
  });
}
