#![allow(dead_code)]

mod hmi;
mod math;
mod render_gl;
mod sys;

use crate::math::{
  colors::{HslColor, HsvColor, RGBAColor, RGBAColorF32, XyzColor},
  vec2::{Vec2F32, Vec2I16},
  vertex_types::VertexPTC,
};

use crate::{
  hmi::{
    base::{AntialiasingType, ConvertConfig, DrawNullTexture, GenericHandle},
    commands::{
      CmdArc, CmdCircle, CmdCircleFilled, CmdPolygon, CmdPolyline, CmdText,
      CmdTriangleFilled, Command,
    },
    text_engine::{
      Font, FontAtlas, FontConfig, FontConfigBuilder, TTFDataSource,
    },
    vertex_output::{DrawCommand, DrawIndexType, DrawList},
  },
  sys::memory_mapped_file::MemoryMappedFile,
};

use glfw::{Action, Context, Key, WindowHint};

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
  let font_atlas = FontAtlas::new(300)
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

  window.set_key_polling(true);
  window.make_current();

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
    gl::Viewport(0, 0, 900, 700);
    let cc = RGBAColorF32::from(HsvColor::new(217f32, 87f32, 46f32));

    println!("{:?}", cc);
    // println!("{}", 7.4 % 6.0);

    // RGBAColorF32::new(0.85f32, 0.15f32, 0.15f32);
    gl::ClearColor(cc.r, cc.g, cc.b, cc.a);
  }

  let world_view_prof_mtx =
    orthographic_projection(0_f32, 0_f32, 900_f32, 700_f32, 0_f32, 1_f32);

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

  let mut drawlist = DrawList::new(
    convert_cfg,
    &mut buff_draw_commands,
    &mut buff_vertices,
    &mut buff_indices,
    AntialiasingType::On,
    AntialiasingType::Off,
  );

  let mut commands = Vec::<Command>::new();

  // let polygon_pts = vec![
  //   Vec2I16::new(100, 100),
  //   Vec2I16::new(300, 100),
  //   Vec2I16::new(500, 200),
  //   Vec2I16::new(300, 300),
  //   Vec2I16::new(100, 300),
  // ];

  // let cmd_polygon = CmdPolygon {
  //   color:          RGBAColor::new(0, 255, 255),
  //   points:         polygon_pts.clone(),
  //   line_thickness: 2,
  // };

  // commands.push(Command::Polygon(cmd_polygon));

  // let cmd_polyline = CmdPolyline {
  //   color:          RGBAColor::new(255, 0, 0),
  //   line_thickness: 2,
  //   points:         polygon_pts
  //     .iter()
  //     .map(|v| *v + Vec2I16::new(400, 300))
  //     .collect(),
  // };
  // commands.push(Command::Polyline(cmd_polyline));

  // let cmd_circle = CmdCircleFilled {
  //   x:     400,
  //   y:     400,
  //   w:     300,
  //   h:     300,
  //   color: RGBAColor::new(128, 255, 64),
  // };
  // commands.push(Command::CircleFilled(cmd_circle));

  // let cmd_circle = CmdCircle {
  //   x:              400,
  //   y:              400,
  //   line_thickness: 2,
  //   w:              100,
  //   h:              100,
  //   color:          RGBAColor::new(64, 128, 255),
  // };
  // commands.push(Command::Circle(cmd_circle));

  // let triangle = CmdTriangleFilled {
  //   a:     Vec2I16::new(0, 500),
  //   b:     Vec2I16::new(200, 100),
  //   c:     Vec2I16::new(400, 500),
  //   color: RGBAColor::new(0, 255, 0),
  // };
  // commands.push(Command::TriangleFilled(triangle));

  // let cmd_arc = CmdArc {
  //   cx:             400,
  //   cy:             100,
  //   r:              100,
  //   line_thickness: 3,
  //   a:              [0_f32, -std::f32::consts::PI],
  //   color:          RGBAColor::new(255, 64, 32),
  // };
  // commands.push(Command::Arc(cmd_arc));

  let mut fonts = vec![];
  let mut font_atlas = FontAtlas::new(300)
    .ok_or("Failed to create font atlas")
    .and_then(|mut atlas_builder| {
      let cfg = FontConfigBuilder::new().size(14f32).build();

      let _f01 = atlas_builder
        .add_font(
          &cfg,
          TTFDataSource::File(std::path::PathBuf::from("DroidSans.ttf")),
        )
        .expect("Failed to load ttf file!");

      fonts.push(_f01);

      let cfg = FontConfigBuilder::new().size(32f32).build();
      let _f02 = atlas_builder
        .add_font(
          &cfg,
          TTFDataSource::File(std::path::PathBuf::from("Babylon5.ttf")),
        )
        .expect("Failed to load ttf file!");

      fonts.push(_f02);

      atlas_builder
        .build(|width: u32, height: u32, pixels: &[u8]| {
          write_atlas_png(width, height, pixels);

          let glyphs_texture = unsafe {
            let mut glyphs_texture: gl::types::GLuint = 0;
            gl::CreateTextures(
              gl::TEXTURE_2D,
              1,
              &mut glyphs_texture as *mut _,
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
        .and_then(|_| Ok(atlas_builder))
    })
    .expect("Failed to initialize font engine!");

  // let cfg = FontConfigBuilder::new().size(14f32).build();

  // let _f01 = font_atlas
  //   .add_font(
  //     &cfg,
  //     TTFDataSource::File(std::path::PathBuf::from("DroidSans.ttf")),
  //   )
  //   .expect("Failed to load ttf file!");

  fn write_string(
    font: &Font,
    fg: RGBAColor,
    bk: RGBAColor,
    x: i16,
    y: i16,
    w: u16,
    h: u16,
    text: &str,
    lst: &mut Vec<Command>,
  ) {
    let text_cmd = CmdText {
      font: *font,
      background: bk,
      foreground: fg,
      x,
      y,
      w,
      h,
      height: 0f32,
      text: text.to_owned(),
    };

    lst.push(Command::Text(text_cmd));
  }

  write_string(
    &fonts[0],
    RGBAColor::new(255, 0, 255),
    RGBAColor::new(0, 255, 0),
    100,
    100,
    400,
    200,
    "Some text here",
    &mut commands,
  );

  drawlist.convert(&commands);

  // drawlist.stroke_poly_line(
  //     &poly_pts,
  //     RGBAColor::new(255, 0, 0, 255),
  //     DrawListStroke::Closed,
  //     4_f32,
  //     AntialiasingType::Off,
  // );

  // let endpts = [Vec2F32::new(50_f32, 50_f32)];
  // let translation = Vec2F32::new(200_f32, 200_f32);
  // let poly_moved = endpts
  //     .iter()
  //     .chain(poly_pts.iter())
  //     .map(|&input_vtx| input_vtx + translation)
  //     .collect::<Vec<_>>();

  // println!("{:?}", poly_moved);

  // drawlist.fill_poly_convex(
  //     &poly_moved,
  //     RGBAColor::new(255, 255, 0, 255),
  //     AntialiasingType::Off,
  // );

  fn slice_bytes_size<T: Sized>(s: &[T]) -> gl::types::GLsizeiptr {
    (s.len() * std::mem::size_of::<T>()) as gl::types::GLsizeiptr
  }

  let nk_vbuff = unsafe {
    let mut buffid: gl::types::GLuint = 0;
    gl::CreateBuffers(1, &mut buffid as *mut _);
    gl::NamedBufferStorage(
      buffid,
      slice_bytes_size(&buff_vertices),
      buff_vertices.as_ptr() as *const gl::types::GLvoid,
      0,
    );
    buffid
  };

  let nk_ibuff = unsafe {
    let mut buffid: gl::types::GLuint = 0;
    gl::CreateBuffers(1, &mut buffid as *mut _);
    gl::NamedBufferStorage(
      buffid,
      slice_bytes_size(&buff_indices),
      buff_indices.as_ptr() as *const gl::types::GLvoid,
      0,
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

  while !window.should_close() {
    glfw.poll_events();
    for (_, event) in glfw::flush_messages(&events) {
      match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
          window.set_should_close(true)
        }
        _ => {}
      }
    }

    let (wnd_w, wnd_h) = window.get_size();
    let (dpy_w, dpy_h) = window.get_framebuffer_size();
    let (fb_scale_x, fb_scale_y) =
      (dpy_w as f32 / wnd_w as f32, dpy_h as f32 / wnd_h as f32);

    unsafe {
      gl::Viewport(0, 0, wnd_w, wnd_h);
      gl::Clear(gl::COLOR_BUFFER_BIT);
      gl::BindVertexArray(nk_vao);
    }

    shader_program.set_used();

    unsafe {
      gl::ProgramUniformMatrix4fv(
        shader_program.id(),
        0,
        1,
        gl::FALSE,
        world_view_prof_mtx.as_ptr() as *const _,
      );

      let mut offset = 0;
      buff_draw_commands.iter().for_each(|cmd| {
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
    }

    window.swap_buffers();
  }
}
