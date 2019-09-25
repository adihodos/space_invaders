#![allow(dead_code)]

mod hmi;
mod math;
mod render_gl;
mod sys;

use crate::math::{
  colors::{HslColor, HsvColor, RGBAColor, RGBAColorF32, XyzColor},
  rectangle::RectangleF32,
  vec2::{Vec2F32, Vec2I16},
  vertex_types::VertexPTC,
};

use crate::{
  hmi::{
    base::{
      AntialiasingType, ConvertConfig, DrawNullTexture, GenericHandle,
      TextAlign,
    },
    panel::PanelFlags,
    style::SymbolType,
    text_engine::{
      Font, FontAtlas, FontAtlasBuilder, FontConfig, FontConfigBuilder,
      TTFDataSource,
    },
    ui_context::UiContext,
    vertex_output::{DrawCommand, DrawIndexType},
  },
  render_gl::OpenGLStateSaveSetRestore,
  sys::memory_mapped_file::MemoryMappedFile,
};

use glfw::{Action, Context, Key, WindowHint};

fn slice_bytes_size<T: Sized>(s: &[T]) -> gl::types::GLsizeiptr {
  (s.len() * std::mem::size_of::<T>()) as gl::types::GLsizeiptr
}

#[rustfmt::skip]
fn orthographic_projection(
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    near: f32,
    far: f32,
) -> Vec<f32> {
    let width = right - left;
    let height = top - bottom;    

    vec![
        2_f32 / width, 0_f32, 0_f32, 0_f32,
        0_f32, 2_f32 / height, 0_f32, 0_f32,
        0_f32, 0_f32, -2_f32 / (far + near), 0_f32,
        -(right + left) / width, -(top + bottom) / height, -(far + near) / (far - near), 1_f32
    ]
}

#[rustfmt::skip]
fn ortho_symm(right : f32, top : f32, near : f32, far : f32) -> Vec<f32> {
    vec![
      1_f32 / right, 0_f32, 0_f32, 0_f32,
      0_f32, 1_f32 / top, 0_f32, 0_f32,
      0_f32, 0_f32, -2_f32 / (far - near), -(far + near) / (far - near),
      0_f32, 0_f32, 0_f32, 1_f32
    ]
}

fn write_atlas_png(width: u32, height: u32, pixels: &[u8]) {
  // For reading and opening files
  use std::{fs::File, io::BufWriter, path::Path};
  // To use encoder.set()
  use png::HasParameters;

  let path = Path::new(r"packed_rects.png");
  let file = File::create(path).unwrap();
  let ref mut w = BufWriter::new(file);

  let mut encoder = png::Encoder::new(w, width, height); // Width is 2 pixels and height is 1.
  encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
  let mut writer = encoder.write_header().unwrap();

  writer.write_image_data(pixels).unwrap();
}

fn test_font_atlas() {
  let font_atlas = FontAtlasBuilder::new(300)
    .ok_or("Failed to create font atlas")
    .and_then(|mut atlas_builder| {
      let cfg = FontConfigBuilder::new()
        .size(24f32)
        .add_glyph_range(FontConfigBuilder::default_cyrillic_glyph_ranges())
        .build();

      let _f01 = atlas_builder
        .add_font(
          &cfg,
          TTFDataSource::File(std::path::PathBuf::from("DroidSans.ttf")),
        )
        .expect("Failed to load ttf file!");

      let cfg = FontConfigBuilder::new().size(64f32).build();
      let _f02 = atlas_builder
        .add_font(
          &cfg,
          TTFDataSource::File(std::path::PathBuf::from("Babylon5.ttf")),
        )
        .expect("Failed to load ttf file!");

      atlas_builder.build(|width: u32, height: u32, pixels: &[u8]| {
        write_atlas_png(width, height, pixels);
        Some((
          GenericHandle::Id(1),
          DrawNullTexture {
            texture: GenericHandle::Id(2),
            uv:      Vec2F32::new(0f32, 0f32),
          },
        ))
      })
    })
    .expect("Failed to initialize font engine!");
}

fn main() {
  // test_font_atlas();

  let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
  glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
  glfw.window_hint(WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
  glfw.window_hint(WindowHint::ContextVersion(4, 5));

  let (mut window, events) = glfw
    .create_window(
      1200,
      1024,
      "Hello this is window",
      glfw::WindowMode::Windowed,
    )
    .expect("Failed to create GLFW window.");

  window.make_current();
  // window.set_all_polling(true);
  window.set_key_polling(true);
  // window.set_framebuffer_size_polling(true);

  gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

  // set up shader program
  use std::ffi::CString;
  let vert_shader = render_gl::Shader::from_vert_source(
    &CString::new(include_str!("render.vert")).unwrap(),
  )
  .unwrap();

  let frag_shader = render_gl::Shader::from_frag_source(
    &CString::new(include_str!("render.frag")).unwrap(),
  )
  .unwrap();

  let shader_program =
    render_gl::Program::from_shaders(&[vert_shader, frag_shader]).unwrap();

  let white_pixel_tex = unsafe {
    let mut texid: gl::types::GLuint = 0;
    gl::CreateTextures(gl::TEXTURE_2D, 1, &mut texid as *mut _);
    // 1x1 white texture
    let texture_data = [255u8, 255u8, 255u8, 255u8];
    gl::TextureStorage2D(texid, 1, gl::RGBA8, 1, 1);
    gl::TextureSubImage2D(
      texid,
      0,
      0,
      0,
      1,
      1,
      gl::RGBA,
      gl::UNSIGNED_BYTE,
      texture_data.as_ptr() as *const gl::types::GLvoid,
    );

    texid
  };

  let tex_sampler = unsafe {
    let mut smpid: gl::types::GLuint = 0;
    gl::CreateSamplers(1, &mut smpid as *mut _);
    gl::SamplerParameteri(
      smpid,
      gl::TEXTURE_MIN_FILTER,
      gl::LINEAR as gl::types::GLint,
    );
    gl::SamplerParameteri(
      smpid,
      gl::TEXTURE_MAG_FILTER,
      gl::LINEAR as gl::types::GLint,
    );

    smpid
  };

  unsafe {
    let cc = RGBAColorF32::from(HsvColor::new(217f32, 87f32, 46f32));
    gl::ClearColor(cc.r, cc.g, cc.b, cc.a);
  }

  unsafe {
    gl::BindTextures(0, 1, &white_pixel_tex as *const _);
    gl::BindSamplers(0, 1, &tex_sampler as *const _);
  }

  // main loop
  let null_tex = DrawNullTexture {
    texture: GenericHandle::Id(white_pixel_tex),
    uv:      Vec2F32::new(0_f32, 0_f32),
  };

  let mut buff_vertices = Vec::<VertexPTC>::new();
  let mut buff_indices = Vec::<DrawIndexType>::new();
  let mut buff_draw_commands = Vec::<DrawCommand>::new();

  let convert_cfg = ConvertConfig {
    global_alpha:         1_f32,
    line_aa:              AntialiasingType::On,
    shape_aa:             AntialiasingType::On,
    circle_segment_count: 22,
    arc_segment_count:    22,
    curve_segment_count:  22,
    null:                 null_tex,
    vertex_layout:        vec![],
    vertex_size:          std::mem::size_of::<VertexPTC>(),
  };

  let mut fonts = vec![];
  let font_atlas = FontAtlasBuilder::new(96)
    .ok_or("Failed to create font atlas")
    .and_then(|mut atlas_builder| {
      let cfg = FontConfigBuilder::new().size(24f32).build();

      let _f01 = atlas_builder
        .add_font(
          &cfg,
          TTFDataSource::File(std::path::PathBuf::from("DroidSans.ttf")),
        )
        .expect("Failed to load ttf file!");

      fonts.push(_f01);

      let cfg = FontConfigBuilder::new().size(14f32).build();
      let _f02 = atlas_builder
        .add_font(
          &cfg,
          TTFDataSource::File(std::path::PathBuf::from("Babylon5.ttf")),
        )
        .expect("Failed to load ttf file!");

      fonts.push(_f02);

      atlas_builder.build(|width: u32, height: u32, pixels: &[u8]| {
        write_atlas_png(width, height, pixels);

        let glyphs_texture = unsafe {
          let mut glyphs_texture: gl::types::GLuint = 0;
          gl::CreateTextures(gl::TEXTURE_2D, 1, &mut glyphs_texture as *mut _);
          gl::TextureStorage2D(
            glyphs_texture,
            1,
            gl::RGBA8,
            width as gl::types::GLsizei,
            height as gl::types::GLsizei,
          );
          gl::TextureSubImage2D(
            glyphs_texture,
            0,
            0,
            0,
            width as gl::types::GLsizei,
            height as gl::types::GLsizei,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            pixels.as_ptr() as *const u8 as *const gl::types::GLvoid,
          );

          glyphs_texture
        };

        Some((GenericHandle::Id(glyphs_texture), null_tex))
      })
    })
    .expect("Failed to initialize font engine!");

  let nk_vbuff = unsafe {
    let mut buffid: gl::types::GLuint = 0;
    gl::CreateBuffers(1, &mut buffid as *mut _);
    gl::NamedBufferStorage(
      buffid,
      (2048 * std::mem::size_of::<VertexPTC>()) as isize,
      std::ptr::null_mut(),
      gl::MAP_WRITE_BIT,
    );
    buffid
  };

  let nk_ibuff = unsafe {
    let mut buffid: gl::types::GLuint = 0;
    gl::CreateBuffers(1, &mut buffid as *mut _);
    gl::NamedBufferStorage(
      buffid,
      (2048 * std::mem::size_of::<DrawIndexType>()) as isize,
      std::ptr::null_mut(),
      gl::MAP_WRITE_BIT,
    );

    buffid
  };

  let nk_vao = unsafe {
    let mut vao: gl::types::GLuint = 0;
    gl::CreateVertexArrays(1, &mut vao as *mut _);
    gl::VertexArrayVertexBuffer(
      vao,
      0,
      nk_vbuff,
      0,
      std::mem::size_of::<VertexPTC>() as gl::types::GLsizei,
    );
    gl::VertexArrayElementBuffer(vao, nk_ibuff);

    gl::EnableVertexArrayAttrib(vao, 0);
    gl::VertexArrayAttribBinding(vao, 0, 0);
    gl::VertexArrayAttribFormat(vao, 0, 2, gl::FLOAT, gl::FALSE, 0);

    gl::EnableVertexArrayAttrib(vao, 1);
    gl::VertexArrayAttribBinding(vao, 1, 0);
    gl::VertexArrayAttribFormat(vao, 1, 2, gl::FLOAT, gl::FALSE, 8);

    gl::EnableVertexArrayAttrib(vao, 2);
    gl::VertexArrayAttribBinding(vao, 2, 0);
    gl::VertexArrayAttribFormat(vao, 2, 4, gl::FLOAT, gl::FALSE, 16);

    vao
  };

  // let mut ui_ctx = UiContext::new(
  //   fonts[0],
  //   convert_cfg,
  //   AntialiasingType::Off,
  //   AntialiasingType::Off,
  // );

  // use crate::hmi::commands::CommandBuffer;
  // let mut cmd_buff = CommandBuffer::new(None, 64);

  // let btn_bounds = RectangleF32 {
  //   x: 20f32,
  //   y: 20f32,
  //   w: 255f32,
  //   h: 64f32,
  // };

  // cmd_buff.stroke_rect(btn_bounds, 0f32, 2f32, RGBAColor::new(200, 200,
  // 200));

  // let content_bounds = RectangleF32::shrink(&btn_bounds, 2f32);
  // cmd_buff.fill_rect(content_bounds, 0f32, RGBAColor::new(64, 64, 64));

  // cmd_buff.draw_text(
  //   content_bounds,
  //   "Demo",
  //   fonts[0],
  //   RGBAColor::new(20, 20, 20),
  //   RGBAColor::new(255, 0, 0),
  // );

  use crate::hmi::commands::*;
  let mut cmd_buff: Vec<Command> = vec![];
  // cmd_buff.push(Command::RectFilled(CmdRectFilled {
  //   rounding: 0,
  //   x:        50,
  //   y:        50,
  //   w:        230,
  //   h:        41,
  //   color:    RGBAColor {
  //     r: 40,
  //     g: 40,
  //     b: 40,
  //     a: 255,
  //   },
  // }));

  cmd_buff.push(Command::Text(CmdText {
    font:       fonts[0],
    background: RGBAColor {
      r: 40,
      g: 40,
      b: 40,
      a: 255,
    },
    foreground: RGBAColor {
      r: 175,
      g: 175,
      b: 175,
      a: 255,
    },
    x:          58,
    y:          58,
    w:          86,
    h:          16,
    height:     24.0,
    text:       String::from("Demo"),
  }));

  cmd_buff.push(Command::RectFilled(CmdRectFilled {
    rounding: 0,
    x:        50,
    y:        90,
    w:        230,
    h:        210,
    color:    RGBAColor {
      r: 45,
      g: 45,
      b: 45,
      a: 255,
    },
  }));

  //   cmd_buff.push(Command::Rect(CmdRect {
  //   rounding: 0,
  //   line_thickness: 1,
  //   x:        50,
  //   y:        90,
  //   w:        230,
  //   h:        210,
  //   color:    RGBAColor {
  //     r: 45,
  //     g: 45,
  //     b: 45,
  //     a: 255,
  //   },
  // }));

  // cmd_buff.push(Command::Scissor(CmdScissor {
  //   x: -8192,
  //   y: -8192,
  //   w: 16834,
  //   h: 16834,
  // }));
  // cmd_buff.push(Command::Scissor(CmdScissor {
  //   x: -8192,
  //   y: -8192,
  //   w: 16834,
  //   h: 16834,
  // }));

  use crate::hmi::vertex_output::DrawList;
  let mut draw_list =
    DrawList::new(convert_cfg, AntialiasingType::Off, AntialiasingType::Off);

  draw_list.convert_commands_range(
    &cmd_buff,
    &mut buff_vertices,
    &mut buff_indices,
    &mut buff_draw_commands,
  );

  while !window.should_close() {
    glfw.poll_events();
    // pass input to UI
    // ui_ctx.input_mut().begin();

    for (_, event) in glfw::flush_messages(&events) {
      match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
          println!("ESC pressed -> quitting ...");
          window.set_should_close(true)
        }

        _ => {}
      }
    }

    // ui_ctx.input_mut().end();

    // UI here
    // ui_ctx.begin(
    //   "Demo",
    //   RectangleF32::new(50f32, 50f32, 230f32, 250f32),
    //   PanelFlags::WindowTitle.into(),
    // );

    // ui_ctx.end();

    // buff_draw_commands.clear();
    // buff_indices.clear();
    // buff_vertices.clear();

    // ui_ctx.convert(
    //   &mut buff_draw_commands,
    //   &mut buff_vertices,
    //   &mut buff_indices,
    // );

    unsafe {
      // upload data to GPU
      let mem_addr_vb = gl::MapNamedBuffer(nk_vbuff, gl::WRITE_ONLY);
      if !mem_addr_vb.is_null() {
        std::ptr::copy_nonoverlapping(
          buff_vertices.as_ptr(),
          mem_addr_vb as *mut VertexPTC,
          buff_vertices.len(),
        );
        gl::UnmapNamedBuffer(nk_vbuff);
      }

      let mem_addr_ib = gl::MapNamedBuffer(nk_ibuff, gl::WRITE_ONLY);
      if !mem_addr_ib.is_null() {
        std::ptr::copy_nonoverlapping(
          buff_indices.as_ptr(),
          mem_addr_ib as *mut DrawIndexType,
          buff_indices.len(),
        );
        gl::UnmapNamedBuffer(nk_ibuff);
      }
    }

    let (wnd_w, wnd_h) = window.get_size();
    let (dpy_w, dpy_h) = window.get_framebuffer_size();
    let (fb_scale_x, fb_scale_y) =
      (dpy_w as f32 / wnd_w as f32, dpy_h as f32 / wnd_h as f32);

    unsafe {
      gl::ViewportIndexedf(0, 0f32, 0f32, dpy_w as f32, dpy_h as f32);
      gl::Clear(gl::COLOR_BUFFER_BIT);
      gl::BindVertexArray(nk_vao);
    }

    shader_program.set_used();

    let world_view_prof_mtx = orthographic_projection(
      0_f32,
      0_f32,
      dpy_w as f32,
      dpy_h as f32,
      0_f32,
      1_f32,
    );

    unsafe {
      gl::ProgramUniformMatrix4fv(
        shader_program.id(),
        0,
        1,
        gl::FALSE,
        world_view_prof_mtx.as_ptr() as *const _,
      );

      let mut offset = 0;
      let _gl_state_save_restore = OpenGLStateSaveSetRestore::new();

      buff_draw_commands.iter().for_each(|cmd| {
        // dbg!(cmd);
        if cmd.element_count == 0 {
          return;
        }

        match cmd.texture {
          GenericHandle::Id(tex_id) => {
            gl::BindTextureUnit(0, tex_id);
          }
          _ => {}
        }

        use gl::types::{GLint, GLsizei, GLvoid};

        gl::Scissor(
          (cmd.clip_rect.x * fb_scale_x) as GLint,
          ((wnd_h as f32 - (cmd.clip_rect.y + cmd.clip_rect.h)) * fb_scale_y)
            as GLint,
          (cmd.clip_rect.w * fb_scale_x) as GLint,
          (cmd.clip_rect.h * fb_scale_y) as GLint,
        );

        gl::DrawElements(
          gl::TRIANGLES,
          cmd.element_count as GLsizei,
          gl::UNSIGNED_SHORT,
          offset as *const GLvoid,
        );
        offset += cmd.element_count;
      });

      // ui_ctx.clear();
    }

    window.swap_buffers();
  }
}
