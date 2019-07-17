use gl;
use std::{
  self,
  ffi::{CStr, CString},
};

/// Saves the OpenGL state on creation, enables blending and restores the saved
/// state when dropped.
pub struct OpenGLStateSaveSetRestore {
  last_blend_src:      gl::types::GLint,
  last_blend_dst:      gl::types::GLint,
  last_blend_eq_rgb:   gl::types::GLint,
  last_blend_eq_alpha: gl::types::GLint,
  blend_enabled:       bool,
  cullface_enabled:    bool,
  depth_enabled:       bool,
  scissors_enabled:    bool,
}

impl OpenGLStateSaveSetRestore {
  pub fn new() -> OpenGLStateSaveSetRestore {
    unsafe {
      let mut st: OpenGLStateSaveSetRestore = ::std::mem::zeroed();
      gl::GetIntegerv(gl::BLEND_SRC, &mut st.last_blend_src as *mut _);
      gl::GetIntegerv(gl::BLEND_DST, &mut st.last_blend_dst as *mut _);
      gl::GetIntegerv(
        gl::BLEND_EQUATION_RGB,
        &mut st.last_blend_eq_rgb as *mut _,
      );
      gl::GetIntegerv(
        gl::BLEND_EQUATION_ALPHA,
        &mut st.last_blend_eq_alpha as *mut _,
      );
      st.blend_enabled = gl::IsEnabled(gl::BLEND) != gl::FALSE;
      st.cullface_enabled = gl::IsEnabled(gl::CULL_FACE) != gl::FALSE;
      st.depth_enabled = gl::IsEnabled(gl::DEPTH_TEST) != gl::FALSE;
      st.scissors_enabled = gl::IsEnabled(gl::SCISSOR_TEST) != gl::FALSE;

      gl::Enable(gl::BLEND);
      gl::BlendEquation(gl::FUNC_ADD);
      gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
      gl::Disable(gl::CULL_FACE);
      gl::Disable(gl::DEPTH_TEST);
      gl::Enable(gl::SCISSOR_TEST);

      st
    }
  }
}

impl Drop for OpenGLStateSaveSetRestore {
  fn drop(&mut self) {
    unsafe {
      gl::BlendEquationSeparate(
        self.last_blend_eq_rgb as u32,
        self.last_blend_eq_alpha as u32,
      );
      gl::BlendFunc(self.last_blend_src as u32, self.last_blend_dst as u32);

      if !self.blend_enabled {
        gl::Disable(gl::BLEND)
      }

      if self.cullface_enabled {
        gl::Enable(gl::CULL_FACE)
      }

      if self.depth_enabled {
        gl::Enable(gl::DEPTH_TEST)
      }

      if !self.scissors_enabled {
        gl::Disable(gl::SCISSOR_TEST);
      }
    }
  }
}

pub struct Program {
  id: gl::types::GLuint,
}

impl Program {
  pub fn from_shaders(shaders: &[Shader]) -> Result<Program, String> {
    let program_id = unsafe { gl::CreateProgram() };

    for shader in shaders {
      unsafe {
        gl::AttachShader(program_id, shader.id());
      }
    }

    unsafe {
      gl::LinkProgram(program_id);
    }

    let mut success: gl::types::GLint = 1;
    unsafe {
      gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
    }

    if success == 0 {
      let mut len: gl::types::GLint = 0;
      unsafe {
        gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
      }

      let error = create_whitespace_cstring_with_len(len as usize);
      unsafe {
        gl::GetProgramInfoLog(
          program_id,
          len,
          std::ptr::null_mut(),
          error.as_ptr() as *mut gl::types::GLchar,
        );
      }

      return Err(error.to_string_lossy().into_owned());
    }

    for shader in shaders {
      unsafe {
        gl::DetachShader(program_id, shader.id());
      }
    }

    Ok(Program { id: program_id })
  }

  pub fn id(&self) -> gl::types::GLuint {
    self.id
  }

  pub fn set_used(&self) {
    unsafe {
      gl::UseProgram(self.id);
    }
  }
}

impl Drop for Program {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteProgram(self.id);
    }
  }
}

pub struct Shader {
  id: gl::types::GLuint,
}

impl Shader {
  pub fn from_source(
    source: &CStr,
    kind: gl::types::GLenum,
  ) -> Result<Shader, String> {
    let id = shader_from_source(source, kind)?;
    Ok(Shader { id })
  }

  pub fn from_vert_source(source: &CStr) -> Result<Shader, String> {
    Shader::from_source(source, gl::VERTEX_SHADER)
  }

  pub fn from_frag_source(source: &CStr) -> Result<Shader, String> {
    Shader::from_source(source, gl::FRAGMENT_SHADER)
  }

  pub fn id(&self) -> gl::types::GLuint {
    self.id
  }
}

impl Drop for Shader {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteShader(self.id);
    }
  }
}

fn shader_from_source(
  source: &CStr,
  kind: gl::types::GLenum,
) -> Result<gl::types::GLuint, String> {
  let id = unsafe { gl::CreateShader(kind) };
  unsafe {
    gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
    gl::CompileShader(id);
  }

  let mut success: gl::types::GLint = 1;
  unsafe {
    gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
  }

  if success == 0 {
    let mut len: gl::types::GLint = 0;
    unsafe {
      gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
    }

    let error = create_whitespace_cstring_with_len(len as usize);

    unsafe {
      gl::GetShaderInfoLog(
        id,
        len,
        std::ptr::null_mut(),
        error.as_ptr() as *mut gl::types::GLchar,
      );
    }

    return Err(error.to_string_lossy().into_owned());
  }

  Ok(id)
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
  // allocate buffer of correct size
  let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
  // fill it with len spaces
  buffer.extend([b' '].iter().cycle().take(len));
  // convert buffer to CString
  unsafe { CString::from_vec_unchecked(buffer) }
}
