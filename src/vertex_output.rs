use std::ffi::c_void;

use super::commands::*;
use super::types::*;
pub type DrawIndexType = u16;

#[derive(Copy, Debug, Clone)]
pub enum DrawListStroke {
  Open,
  Closed,
}

// impl DrawVertexLayoutElement {
//   pub fn is_end_of_layout(&self) -> bool {
//     self.attribute == DrawVertexLayoutAttribute::Count && self.format == DrawVertexLayoutFormat::FormatCount
//   }
// }

#[derive(Copy, Debug, Clone)]
pub struct DrawCommand {
  pub element_count: u32,
  pub clip_rect: Option<RectangleF32>,
  pub texture: GenericHandle,
}

#[derive(Debug)]
pub struct DrawList<'a, 'b, 'c> {
  clip_rect: Option<RectangleF32>,
  circle_vtx: Vec<Vec2F32>,
  config: ConvertConfig,
  buffer: &'a mut Vec<DrawCommand>,
  vertices: &'b mut Vec<VertexPTC>,
  elements: &'c mut Vec<DrawIndexType>,
  path_count: u32,
  path_offset: u32,
  line_aa: AntialiasingType,
  shape_aa: AntialiasingType,
}

impl<'a, 'b, 'c> DrawList<'a, 'b, 'c> {
  pub fn new(
    config: ConvertConfig,
    cmds: &'a mut Vec<DrawCommand>,
    vertices: &'b mut Vec<VertexPTC>,
    elements: &'c mut Vec<DrawIndexType>,
    line_aa: AntialiasingType,
    shape_aa: AntialiasingType,
  ) -> Self {
    const GEN_CIRCLE_VERTICES_COUNT: i32 = 12;

    DrawList {
      clip_rect: None,
      circle_vtx: (0..GEN_CIRCLE_VERTICES_COUNT)
        .map(|idx| {
          let a = idx as f32 / (GEN_CIRCLE_VERTICES_COUNT as f32 * 2_f32 * std::f32::consts::PI);
          Vec2F32::new(a.cos(), a.sin())
        })
        .collect(),
      config,
      buffer: cmds,
      vertices,
      elements,
      path_count: 0,
      path_offset: 0,
      line_aa,
      shape_aa,
    }
  }

  fn push_command(&mut self, clip: Option<RectangleF32>, texture: GenericHandle) {
    let cmd = DrawCommand {
      element_count: 0,
      clip_rect: clip,
      texture,
    };

    self.buffer.push(cmd);
    self.clip_rect = clip;
  }

  fn add_clip(&mut self, rect: RectangleF32) {
    if self.buffer.is_empty() {
      self.push_command(Some(rect), self.config.null.texture);
      return;
    }

    let prev_cmd_texture = self
      .buffer
      .last_mut()
      .and_then(|last_cmd| {
        if last_cmd.element_count == 0 {
          last_cmd.clip_rect = Some(rect);
        }

        Some(last_cmd.texture)
      })
      .unwrap();

    self.push_command(Some(rect), prev_cmd_texture);
  }

  fn push_image(&mut self, texture: GenericHandle) {
    if self.buffer.is_empty() {
      self.push_command(None, texture);
    }

    self
      .buffer
      .last_mut()
      .and_then(|prev_cmd| {
        if prev_cmd.element_count == 0 {
          prev_cmd.texture = texture;
          None
        } else if prev_cmd.texture != texture {
          Some(())
        } else {
          None
        }
      })
      .and_then(|_| {
        self.push_command(self.clip_rect.as_ref().copied(), texture);
        Some(())
      });
  }

  fn draw_vertex_color(attr_ptr: *mut c_void, vals: &[f32], format: DrawVertexLayoutFormat) {
    assert!(format > DrawVertexLayoutFormat::FormatColorBegin);
    assert!(format < DrawVertexLayoutFormat::FormatColorEnd);
    assert!(vals.len() == 4);

    if format <= DrawVertexLayoutFormat::FormatColorBegin
      || format >= DrawVertexLayoutFormat::FormatColorEnd
    {
      return;
    }

    let src_col = RGBAColorF32::new(
      saturate(vals[0]),
      saturate(vals[1]),
      saturate(vals[2]),
      saturate(vals[3]),
    );

    match format {
      DrawVertexLayoutFormat::R8G8B8 | DrawVertexLayoutFormat::R8G8B8A8 => {
        let clr = rgba_color_f32_to_rgba_color(src_col);
        unsafe {
          std::ptr::copy_nonoverlapping(clr.as_slice().as_ptr(), attr_ptr as *mut u8, 4);
        }
      }

      DrawVertexLayoutFormat::B8G8R8A8 => {
        let col = rgba_color_f32_to_rgba_color(src_col);
        let bgra = RGBAColor::new(col.b, col.g, col.r, col.a);
        unsafe {
          std::ptr::copy_nonoverlapping(bgra.as_slice().as_ptr(), attr_ptr as *mut u8, 4);
        }
      }

      DrawVertexLayoutFormat::R16G15B16 => {
        let col = [
          (src_col.r * std::u8::MAX as f32) as u8,
          (src_col.g * std::u8::MAX as f32) as u8,
          (src_col.b * std::u8::MAX as f32) as u8,
        ];
        unsafe {
          std::ptr::copy_nonoverlapping(col.as_ptr(), attr_ptr as *mut u8, col.len());
        }
      }

      DrawVertexLayoutFormat::R16G15B16A16 => {
        let col = [
          (src_col.r * std::u8::MAX as f32) as u8,
          (src_col.g * std::u8::MAX as f32) as u8,
          (src_col.b * std::u8::MAX as f32) as u8,
          (src_col.a * std::u8::MAX as f32) as u8,
        ];
        unsafe {
          std::ptr::copy_nonoverlapping(col.as_ptr(), attr_ptr as *mut u8, col.len());
        }
      }

      DrawVertexLayoutFormat::R32G32B32 => {
        let col = [
          (src_col.r * std::u32::MAX as f32) as u32,
          (src_col.g * std::u32::MAX as f32) as u32,
          (src_col.b * std::u32::MAX as f32) as u32,
        ];
        unsafe {
          std::ptr::copy_nonoverlapping(col.as_ptr(), attr_ptr as *mut u32, col.len());
        }
      }

      DrawVertexLayoutFormat::R32G32B32A32 => {
        let col = [
          (src_col.r * std::u32::MAX as f32) as u32,
          (src_col.g * std::u32::MAX as f32) as u32,
          (src_col.b * std::u32::MAX as f32) as u32,
          (src_col.a * std::u32::MAX as f32) as u32,
        ];
        unsafe {
          std::ptr::copy_nonoverlapping(col.as_ptr(), attr_ptr as *mut u32, col.len());
        }
      }

      DrawVertexLayoutFormat::R32G32B32A32_Float => unsafe {
        std::ptr::copy_nonoverlapping(src_col.as_slice().as_ptr(), attr_ptr as *mut f32, 4);
      },

      DrawVertexLayoutFormat::R32G32B32A32_Double => {
        let col = [
          src_col.r as f64,
          src_col.g as f64,
          src_col.b as f64,
          src_col.a as f64,
        ];
        unsafe {
          std::ptr::copy_nonoverlapping(col.as_ptr(), attr_ptr as *mut f64, col.len());
        }
      }

      DrawVertexLayoutFormat::RGB32 | DrawVertexLayoutFormat::RGBA32 => {
        let col = rgba_color_to_u32(rgba_color_f32_to_rgba_color(src_col));
        unsafe {
          std::ptr::copy_nonoverlapping(&col as *const u32, attr_ptr as *mut u32, 1);
        }
      }

      _ => panic!("Invalid Vertex Layout format"),
    }
  }

  fn draw_vertex_element(attribute: *mut c_void, values: &[f32], format: DrawVertexLayoutFormat) {
    let mut attribute = attribute as *mut u8;

    assert!(
      format < DrawVertexLayoutFormat::FormatColorBegin,
      "Invalid format provided for a value"
    );

    if format >= DrawVertexLayoutFormat::FormatColorBegin {
      return;
    }

    values.iter().for_each(|&val| match format {
      DrawVertexLayoutFormat::Schar => {
        let value = clamp(std::i8::MIN as f32, val, std::i8::MAX as f32) as i8;
        unsafe {
          std::ptr::copy_nonoverlapping(&value as *const i8, attribute as *mut i8, 1);
          attribute = attribute.add(std::mem::size_of::<i8>());
        }
      }

      DrawVertexLayoutFormat::Sshort => {
        let value = clamp(std::i16::MIN as f32, val, std::i16::MAX as f32) as i16;
        unsafe {
          std::ptr::copy_nonoverlapping(&value as *const i16, attribute as *mut i16, 1);
          attribute = attribute.add(std::mem::size_of::<i16>());
        }
      }

      DrawVertexLayoutFormat::Sint => {
        let value = clamp(std::i32::MIN as f32, val, std::i32::MAX as f32) as i32;
        unsafe {
          std::ptr::copy_nonoverlapping(&value as *const i32, attribute as *mut i32, 1);
          attribute = attribute.add(std::mem::size_of::<i32>());
        }
      }

      DrawVertexLayoutFormat::Uchar => {
        let value = clamp(std::u8::MIN as f32, val, std::u8::MAX as f32) as u8;
        unsafe {
          std::ptr::copy_nonoverlapping(&value as *const u8, attribute, 1);
          attribute = attribute.add(std::mem::size_of::<u8>());
        }
      }

      DrawVertexLayoutFormat::Ushort => {
        let value = clamp(std::u16::MIN as f32, val, std::u16::MAX as f32) as u16;
        unsafe {
          std::ptr::copy_nonoverlapping(&value as *const u16, attribute as *mut u16, 1);
          attribute = attribute.add(std::mem::size_of::<u16>());
        }
      }

      DrawVertexLayoutFormat::Uint => {
        let value = clamp(std::u32::MIN as f32, val, std::u32::MAX as f32) as u32;
        unsafe {
          std::ptr::copy_nonoverlapping(&value as *const u32, attribute as *mut u32, 1);
          attribute = attribute.add(std::mem::size_of::<u32>());
        }
      }

      DrawVertexLayoutFormat::Float => unsafe {
        std::ptr::copy_nonoverlapping(&val as *const f32, attribute as *mut f32, 1);
        attribute = attribute.add(std::mem::size_of::<f32>());
      },

      DrawVertexLayoutFormat::Double => {
        let value = val as f64;
        unsafe {
          std::ptr::copy_nonoverlapping(&value as *const f64, attribute as *mut f64, 1);
          attribute = attribute.add(std::mem::size_of::<f64>());
        }
      }

      _ => assert!(false, "Invalid vertex layout format"),
    });
  }

  fn draw_vertex(
    dst: *mut c_void,
    config: &ConvertConfig,
    pos: Vec2F32,
    uv: Vec2F32,
    color: RGBAColorF32,
  ) {
    config.vertex_layout.iter().for_each(|layout_element| {
      let addr = unsafe { (dst as *mut u8).add(layout_element.offset) as *mut c_void };
      match layout_element.attribute {
        DrawVertexLayoutAttribute::Position => {
          DrawList::draw_vertex_element(addr, pos.as_slice(), layout_element.format)
        }
        DrawVertexLayoutAttribute::Texcoord => {
          DrawList::draw_vertex_element(addr, uv.as_slice(), layout_element.format)
        }
        DrawVertexLayoutAttribute::Color => {
          DrawList::draw_vertex_color(addr, color.as_slice(), layout_element.format)
        }
      }
    });
  }

  // fn alloc_vertices(&mut self, vertex_count: usize) {
  //   self
  //     .vertices
  //     .resize_with(vertex_count, std::default::Default::default);
  //   if std::mem::size_of::<DrawIndexType>() == 2 {
  //     assert!(self.vertices.len() < std::u16::MAX as usize, "To many vertices for 16-bit vertex indices. Redefine DrawIndexType as a 32 bit unsigned integer!");
  //   }
  // }
}
