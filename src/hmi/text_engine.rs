#![allow(dead_code)]

use freetype_sys::*;
use std::collections::HashMap;

use crate::{
  hmi::base::{DrawNullTexture, GenericHandle},
  math::{
    colors::RGBAColor, rectangle::RectangleI32, utility::roundup_multiple_of,
    vec2::Vec2F32,
  },
  sys::{
    memory_mapped_file::MemoryMappedFile,
    unique_resource::{ResourceDeleter, UniqueResource},
  },
};

/// Models a single span of gray pixels when rendering a glyph outline.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Span {
  pub x:        i32,
  pub y:        i32,
  pub width:    i32,
  pub coverage: i32,
}

impl Span {
  fn new(x: i32, y: i32, width: i32, coverage: i32) -> Span {
    Span {
      x,
      y,
      width,
      coverage,
    }
  }

  extern "C" fn raster_callback(
    y: i32,
    count: i32,
    spans: *const FT_Span,
    user: *mut libc::c_void,
  ) {
    use std::slice;
    unsafe {
      let spans = slice::from_raw_parts(spans, count as usize);
      let span_coll = user as *mut Vec<Span>;
      spans.iter().for_each(|s| {
        let s = *s;
        (*span_coll).push(Span::new(
          s.x as i32,
          y,
          s.len as i32,
          s.coverage as i32,
        ));
      });
    }
  }

  fn render_spans(
    library: FT_Library,
    outline: &mut FT_Outline,
    spans: &mut Vec<Span>,
  ) {
    unsafe {
      const FT_RASTER_FLAG_AA: FT_Int = 0x1;
      const FT_RASTER_FLAG_DIRECT: FT_Int = 0x2;

      let mut raster_params = std::mem::zeroed::<FT_Raster_Params>();

      raster_params.flags = FT_RASTER_FLAG_AA | FT_RASTER_FLAG_DIRECT;
      raster_params.gray_spans = Span::raster_callback;
      raster_params.user = spans as *mut _ as *mut libc::c_void;

      FT_Outline_Render(
        library,
        outline as *mut _,
        &mut raster_params as *mut _,
      );
    }
  }

  fn bounding_box(spans: &[Span]) -> RectangleI32 {
    assert!(spans.len() != 0);

    let start_bbox =
      RectangleI32::new(spans[0].x, spans[0].y, spans[0].width, 1);

    spans.iter().fold(start_bbox, |current_bbox, span| {
      let span_bbox = RectangleI32::new(span.x, span.y, span.width, 1);
      RectangleI32::union(&current_bbox, &span_bbox)
    })
  }

  fn convert_to_pixels(spans: &[Span]) -> (RectangleI32, Vec<RGBAColor>) {
    let glyph_bbox = Span::bounding_box(&spans);
    let img_width = glyph_bbox.w;
    let img_height = glyph_bbox.h;

    // transform spans to pixels
    let mut glyph_pixels = vec![
      RGBAColor::new_with_alpha(0, 0, 0, 0);
      (img_width * img_height) as usize
    ];

    spans.iter().for_each(|span| {
      for x in 0 .. span.width {
        let dst_idx = ((img_height - 1 - (span.y - glyph_bbox.y)) * img_width
          + span.x
          - glyph_bbox.x
          + x) as usize;
        glyph_pixels[dst_idx] =
          RGBAColor::new_with_alpha(255, 255, 255, span.coverage as u8);
      }
    });

    (glyph_bbox, glyph_pixels)
  }
}

impl ::std::default::Default for Span {
  fn default() -> Span {
    Span::new(0, 0, 0, 0)
  }
}

pub struct FontConfigBuilder {
  size:           f32,
  spacing:        Vec2F32,
  glyph_range:    Vec<std::ops::Range<char>>,
  fallback_glyph: char,
  pixel_snap:     bool,
}

impl FontConfigBuilder {
  pub fn new() -> FontConfigBuilder {
    FontConfigBuilder {
      size:           10f32,
      spacing:        Vec2F32::new(0f32, 0f32),
      glyph_range:    vec![],
      fallback_glyph: '?',
      pixel_snap:     false,
    }
  }

  pub fn default_glyph_ranges() -> Vec<std::ops::Range<char>> {
    vec![std::ops::Range {
      start: 0x0020 as char,
      end:   0x00FF as char,
    }]
  }

  pub fn default_cyrillic_glyph_ranges() -> Vec<std::ops::Range<char>> {
    use std::ops::Range;

    vec![
      Range {
        start: 0x0020 as char,
        end:   0x00FF as char,
      },
      Range {
        start: '\u{400}',
        end:   '\u{52F}',
      },
      Range {
        start: '\u{2DE0}',
        end:   '\u{2DFF}',
      },
      Range {
        start: '\u{A640}',
        end:   '\u{A69F}',
      },
    ]
  }

  pub fn size(&mut self, size: f32) -> &mut Self {
    self.size = size;
    self
  }

  pub fn spacing(&mut self, x: f32, y: f32) -> &mut Self {
    self.spacing.x = x;
    self.spacing.y = y;
    self
  }

  pub fn pixel_snap(&mut self, snap_to_pixel: bool) -> &mut Self {
    self.pixel_snap = snap_to_pixel;
    self
  }

  pub fn add_glyph_range(
    &mut self,
    mut glyph_range: Vec<std::ops::Range<char>>,
  ) -> &mut Self {
    self.glyph_range.append(&mut glyph_range);
    self
  }

  pub fn fallback_glyph(&mut self, glyph: char) -> &mut Self {
    self.fallback_glyph = glyph;
    self
  }

  pub fn build(&mut self) -> FontConfig {
    if self.glyph_range.is_empty() {
      self.add_glyph_range(Self::default_glyph_ranges());
    }

    let glyph_range = std::mem::replace(&mut self.glyph_range, vec![]);

    FontConfig {
      size: self.size,
      spacing: self.spacing,
      glyph_range,
      fallback_glyph: self.fallback_glyph,
      pixel_snap: self.pixel_snap,
    }
  }
}

#[derive(Clone, Debug)]
pub struct FontConfig {
  pub size:           f32,
  pub spacing:        Vec2F32,
  pub glyph_range:    Vec<std::ops::Range<char>>,
  pub fallback_glyph: char,
  pub pixel_snap:     bool,
}

impl FontConfig {
  fn calc_xadvance(&self, advance: i32) -> f32 {
    if self.pixel_snap {
      ((advance as f32 + 0.5f32) as i32) as f32 + self.spacing.x
    } else {
      advance as f32 + self.spacing.x
    }
  }

  fn calc_yadvance(&self, advance: i32) -> f32 {
    if self.pixel_snap {
      ((advance as f32 + 0.5f32) as i32) as f32 + self.spacing.y
    } else {
      advance as f32 + self.spacing.y
    }
  }
}

#[derive(Debug, Copy, Clone)]
pub struct FontMetrics {
  pub size:                f32,
  pub height:              f32,
  pub ascender:            f32,
  pub descender:           f32,
  pub max_advance_width:   f32,
  pub max_advance_height:  f32,
  pub underline_pos:       f32,
  pub underline_thickness: f32,
}

impl FontMetrics {
  /// Extracts face metrics from a Freetype FT_Face handle.
  fn extract(face: FT_Face, font_size: f32, dpi: u32) -> FontMetrics {
    unsafe {
      FT_Set_Char_Size(
        face,
        (font_size as i32 * 64) as FT_F26Dot6,
        0,
        dpi,
        dpi,
      );
    }

    let pixel_size = font_size as i32 * dpi as i32 / 72;
    let units_per_em = unsafe { (*face).units_per_EM as i32 };

    FontMetrics {
      size:                font_size,
      height:              unsafe {
        (*face).height as i32 * pixel_size / units_per_em
      } as f32,
      ascender:            unsafe {
        (*face).ascender as i32 * pixel_size / units_per_em
      } as f32,
      descender:           unsafe {
        (*face).descender.abs() as i32 * pixel_size / units_per_em
      } as f32,
      max_advance_width:   unsafe {
        (*face).max_advance_width as i32 * pixel_size / units_per_em
      } as f32,
      max_advance_height:  unsafe {
        (*face).max_advance_height as i32 * pixel_size / units_per_em
      } as f32,
      underline_pos:       unsafe {
        (*face).underline_position as i32 * pixel_size / units_per_em
      } as f32,
      underline_thickness: unsafe {
        (*face).underline_thickness as i32 * pixel_size / units_per_em
      } as f32,
    }
  }
}

#[derive(Copy, Clone, Debug)]
pub struct Font {
  pub scale: f32,
  glyph_tbl: u32,
  face_tbl:  u32,
  atlas:     *const FontAtlas,
}

impl std::default::Default for Font {
  fn default() -> Self {
    Self {
      scale:     std::f32::MAX,
      glyph_tbl: std::u32::MAX,
      face_tbl:  std::u32::MAX,
      atlas:     std::ptr::null_mut(),
    }
  }
}

impl Font {
  fn atlas_ref(&self) -> Option<&FontAtlas> {
    if self.atlas.is_null() {
      None
    } else {
      Some(unsafe { &*self.atlas })
    }
  }

  pub fn texture(&self) -> GenericHandle {
    self
      .atlas_ref()
      .map_or(GenericHandle::Id(0), |atlas| atlas.glyphs_texture)
  }

  pub fn draw_null_texture(&self) -> DrawNullTexture {
    self
      .atlas_ref()
      .map_or(DrawNullTexture::default(), |atlas| atlas.draw_null_texture)
  }

  pub fn query_glyph(&self, height: f32, codept: char) -> UserFontGlyph {
    self.atlas_ref().map_or(UserFontGlyph::default(), |atlas| {
      atlas.query_font_glyph(self, height, codept)
    })
  }

  pub fn query_text_width(&self, height: f32, text: &str) -> f32 {
    self
      .atlas_ref()
      .map_or(0f32, |atlas| atlas.font_text_width(self, height, text))
  }

  pub fn clamp_text(&self, text: &str, max_width: f32) -> (i32, f32) {
    self
      .atlas_ref()
      .map_or((0, 0f32), |atlas| atlas.clamp_text(self, text, max_width))
  }

  pub fn clamped_string(&self, text: &str, max_width: f32) -> String {
    self.atlas_ref().map_or(String::new(), |atlas| {
      atlas.clamped_string(self, text, max_width)
    })
  }
}

#[derive(Copy, Clone, Debug)]
pub struct UserFontGlyph {
  // texture coordinates
  pub uv: [Vec2F32; 2],
  // offset between top left and glyph
  pub offset: Vec2F32,
  // dimensions
  pub width:  f32,
  pub height: f32,
  // offset to next glyph
  pub xadvance: f32,
}

impl std::default::Default for UserFontGlyph {
  fn default() -> UserFontGlyph {
    UserFontGlyph {
      uv:       [Vec2F32::same(0f32); 2],
      offset:   Vec2F32::same(0f32),
      width:    0f32,
      height:   0f32,
      xadvance: 0f32,
    }
  }
}

#[derive(Copy, Clone, Debug)]
struct FontGlyph {
  pub codepoint:       u32,
  pub xadvance:        f32,
  pub bearing_x:       f32,
  pub bearing_y:       f32,
  pub bbox:            RectangleI32,
  pub uv_top_left:     Vec2F32,
  pub uv_bottom_right: Vec2F32,
}

impl std::default::Default for FontGlyph {
  fn default() -> FontGlyph {
    FontGlyph {
      codepoint:       0,
      xadvance:        0f32,
      bearing_x:       0f32,
      bearing_y:       0f32,
      bbox:            RectangleI32::new(0, 0, 0, 0),
      uv_top_left:     Vec2F32::new(0f32, 0f32),
      uv_bottom_right: Vec2F32::new(0f32, 0f32),
    }
  }
}

macro_rules! freetype_deleter_impl {
  ( $( ($ftwrapper:ident, $fthandle:ty, $del:expr) ),* ) => {

    $(
      #[derive(Default)]
      struct $ftwrapper {}

      impl ResourceDeleter for $ftwrapper {
        type Handle = $fthandle;

        fn is_null(res: &Self::Handle) -> bool {
          *res == std::ptr::null_mut()
        }

        fn null() -> Self::Handle {
          std::ptr::null_mut()
        }

        fn delete(&mut self, res: &mut Self::Handle) {
          unsafe { $del(*res); }
        }
      }
    )*
  };
}

freetype_deleter_impl!(
  (FreetypeLibraryHandle, FT_Library, FT_Done_Library),
  (FreetypeGlyphHandle, FT_Glyph, FT_Done_Glyph),
  (FreetypeFaceHandle, FT_Face, FT_Done_Face),
  (FreetypeStrokerHandle, FT_Stroker, FT_Stroker_Done)
);

pub enum TTFDataSource {
  File(std::path::PathBuf),
  StaticBytes(&'static [u8]),
  OwnedBytes(Vec<u8>),
}

struct BakedGlyph {
  advance_x: f32,
  bearing_x: f32,
  bearing_y: f32,
  // index in the font table
  font:      u32,
  codepoint: u32,
  bbox:      RectangleI32,
  pixels:    Vec<RGBAColor>,
}

impl BakedGlyph {
  fn new(
    codepoint: u32,
    font: u32,
    bearing_x: f32,
    bearing_y: f32,
    advance_x: f32,
    glyph_spans: &[Span],
  ) -> BakedGlyph {
    if glyph_spans.is_empty() {
      // non renderable (space, tab, newline, etc ...)
      BakedGlyph {
        advance_x,
        bearing_x,
        bearing_y,
        codepoint,
        font,
        bbox: RectangleI32::new(0, 0, 0, 0),
        pixels: vec![],
      }
    } else {
      let (glyph_bbox, glyph_pixels) = Span::convert_to_pixels(&glyph_spans);

      BakedGlyph {
        advance_x,
        bearing_x,
        bearing_y,
        codepoint,
        font,
        bbox: glyph_bbox,
        pixels: glyph_pixels,
      }
    }
  }
}

/// Extract all spans from a rasterized glyph.
fn extract_glyph_spans(
  codepoint: u32,
  face: FT_Face,
  lib: FT_Library,
) -> Option<(i32, i32, i32, Vec<Span>)> {
  let ft_glyph_index =
    unsafe { FT_Get_Char_Index(face, codepoint as FT_ULong) };

  if ft_glyph_index == 0 {
    return None;
  }

  const GLYPH_LOAD_FLAGS: i32 = FT_LOAD_NO_BITMAP | FT_LOAD_TARGET_LIGHT;
  let load_result =
    unsafe { FT_Load_Glyph(face, ft_glyph_index, GLYPH_LOAD_FLAGS) };

  if load_result != 0 {
    return None;
  }

  let glyph = unsafe { (*face).glyph };
  let glyph_format = unsafe { (*glyph).format };
  if glyph_format != FT_GLYPH_FORMAT_OUTLINE {
    return None;
  }

  let bearing_x =
    unsafe { ((*(*face).glyph).metrics.horiBearingX >> 6) as i32 };
  let bearing_y =
    unsafe { ((*(*face).glyph).metrics.horiBearingY >> 6) as i32 };
  let advance_x = unsafe { ((*(*face).glyph).advance.x >> 6) as i32 };

  let mut glyph_spans = Vec::<Span>::new();
  let outline_ptr = unsafe { &mut (*glyph).outline };
  Span::render_spans(lib, outline_ptr, &mut glyph_spans);

  Some((bearing_x, bearing_y, advance_x, glyph_spans))
}

/// Packs font glyphs into a rectangular texture.
fn pack_rects(rects: &mut [BakedGlyph]) -> (u32, u32, f32) {
  let (area, max_width) = rects.iter().fold((0, 0), |acc, r| {
    (acc.0 + r.bbox.w * r.bbox.h, acc.1.max(r.bbox.w))
  });

  rects
    .sort_unstable_by(|glyph_a, glyph_b| glyph_b.bbox.h.cmp(&glyph_a.bbox.h));

  let start_with =
    (max_width as f32).max(((area as f32) / 0.95f32).sqrt().ceil());
  let mut spaces =
    vec![RectangleI32::new(0, 0, start_with as i32, std::i32::MAX)];

  let mut width = 0u32;
  let mut height = 0u32;

  (0 .. rects.len()).for_each(|idx_box| {
    // filter non-renderables
    if rects[idx_box].bbox.w == 0 {
      return;
    }
    (0 .. spaces.len()).rev().any(|i| {
      // look for empty spaces that can accomodate the current box
      if rects[idx_box].bbox.w > spaces[i].w
        || rects[idx_box].bbox.h > spaces[i].h
      {
        return false;
      }

      // found the space; add the box to its top-left corner
      // |-------|-------|
      // |  box  |       |
      // |_______|       |
      // |         space |
      // |_______________|

      rects[idx_box].bbox.x = spaces[i].x;
      rects[idx_box].bbox.y = spaces[i].y;

      width =
        width.max(rects[idx_box].bbox.x as u32 + rects[idx_box].bbox.w as u32);
      height =
        height.max(rects[idx_box].bbox.y as u32 + rects[idx_box].bbox.h as u32);

      if rects[idx_box].bbox.w == spaces[i].w
        && rects[idx_box].bbox.h == spaces[i].h
      {
        // space matches the box exactly, remove it
        let last = spaces.pop().unwrap();
        if i < spaces.len() {
          spaces[i] = last;
        }
      } else if rects[idx_box].bbox.h == spaces[i].h {
        // space matches the box width; update it accordingly
        // |---------------|
        // |      box      |
        // |_______________|
        // | updated space |
        // |_______________|
        spaces[i].x += rects[idx_box].bbox.w;
        spaces[i].w -= rects[idx_box].bbox.w;
      } else if rects[idx_box].bbox.w == spaces[i].w {
        // space matches the box width; update it accordingly
        // |---------------|
        // |      box      |
        // |_______________|
        // | updated space |
        // |_______________|
        spaces[i].y += rects[idx_box].bbox.h;
        spaces[i].h -= rects[idx_box].bbox.h;
      } else {
        // otherwise the box splits the space into two spaces
        // |-------|-----------|
        // |  box  | new space |
        // |_______|___________|
        // | updated space     |
        // |___________________|
        spaces.push(RectangleI32::new(
          spaces[i].x + rects[idx_box].bbox.w,
          spaces[i].y,
          spaces[i].w - rects[idx_box].bbox.w,
          rects[idx_box].bbox.h,
        ));
        spaces[i].y += rects[idx_box].bbox.h;
        spaces[i].h -= rects[idx_box].bbox.h;
      }

      true
    });
  });

  (width, height, (area as f32 / (width * height) as f32))
}

pub struct FontAtlasBuilder {
  dpi:               u32,
  baked_glyphs:      Vec<BakedGlyph>,
  glyphs:            Vec<HashMap<u32, FontGlyph>>,
  fonts:             Vec<Font>,
  faces:             Vec<FontMetrics>,
  configs:           Vec<FontConfig>,
  stroker:           UniqueResource<FreetypeStrokerHandle>,
  lib:               UniqueResource<FreetypeLibraryHandle>,
  glyphs_texture:    GenericHandle,
  draw_null_texture: DrawNullTexture,
  atlas:             *mut FontAtlas,
}

impl FontAtlasBuilder {
  /// Creates a new font atlas. Fonts must be added to it before it can be used.
  pub fn new(dpi: u32) -> Option<FontAtlasBuilder> {
    UniqueResource::<FreetypeLibraryHandle>::from_handle({
      let mut ftlib: FT_Library = std::ptr::null_mut();
      unsafe {
        FT_Init_FreeType(&mut ftlib as *mut _);
      }
      ftlib
    })
    .and_then(|ftlib| {
      UniqueResource::<FreetypeStrokerHandle>::from_handle({
        let mut stroker: FT_Stroker = ::std::ptr::null_mut();
        unsafe {
          FT_Stroker_New(*ftlib.handle(), &mut stroker as *mut _);
        }
        stroker
      })
      .and_then(|stroker| {
        Some(FontAtlasBuilder {
          dpi,
          baked_glyphs: Vec::new(),
          glyphs: Vec::new(),
          fonts: Vec::new(),
          faces: Vec::new(),
          configs: Vec::new(),
          stroker,
          lib: ftlib,
          glyphs_texture: GenericHandle::Id(0),
          draw_null_texture: DrawNullTexture {
            texture: GenericHandle::Id(0),
            uv:      Vec2F32::new(0f32, 0f32),
          },
          atlas: Box::into_raw(Box::new(FontAtlas::new())),
        })
      })
    })
  }

  /// Add a font into the atlas from various sources.
  pub fn add_font(
    &mut self,
    font: &FontConfig,
    font_source: TTFDataSource,
  ) -> Option<Font> {
    match font_source {
      TTFDataSource::File(fpath) => {
        MemoryMappedFile::new(&fpath).ok().and_then(|mapped_ttf| {
          self.add_font_from_bytes(font, mapped_ttf.as_slice())
        })
      }
      TTFDataSource::StaticBytes(bytes) => {
        self.add_font_from_bytes(font, bytes)
      }
      TTFDataSource::OwnedBytes(bytes) => {
        self.add_font_from_bytes(font, &bytes)
      }
    }
  }

  /// Builds the font atlas containing all the fonts and glyphs that were added
  /// to it.
  pub fn build<F>(
    &mut self,
    fn_device_glyph_image_upload: F,
  ) -> Result<Box<FontAtlas>, &'static str>
  where
    F: Fn(u32, u32, &[u8]) -> Option<(GenericHandle, DrawNullTexture)>,
  {
    assert!(!self.fonts.is_empty(), "You forgot to add any fonts!");
    assert!(
      !self.baked_glyphs.is_empty(),
      "You forgot to add any fonts!"
    );

    if self.baked_glyphs.is_empty() {
      return Err("no fonts added to the atlas !");
    }

    let (atlas_width, atlas_height, _) = pack_rects(&mut self.baked_glyphs);
    if atlas_width == 0 || atlas_height == 0 {
      return Err("error packing font glyph rects!");
    }

    let (atlas_width, atlas_height) = (
      roundup_multiple_of(atlas_width, 4),
      roundup_multiple_of(atlas_height, 4),
    );

    // build the glyph tables
    let baked_glyphs = std::mem::replace(&mut self.baked_glyphs, vec![]);
    let ipw = 1f32 / (atlas_width) as f32;
    let iph = 1f32 / (atlas_height) as f32;

    baked_glyphs.iter().for_each(|baked_glyph| {
      let font_glyphs_table = &mut self.glyphs[baked_glyph.font as usize];
      let font_metrics = &self.faces[baked_glyph.font as usize];

      let new_glyph = FontGlyph {
        codepoint:       baked_glyph.codepoint,
        xadvance:        baked_glyph.advance_x,
        bearing_x:       baked_glyph.bearing_x,
        bearing_y:       font_metrics.ascender - baked_glyph.bearing_y,
        bbox:            RectangleI32::new(
          0,
          0,
          baked_glyph.bbox.w,
          baked_glyph.bbox.h,
        ),
        uv_top_left:     Vec2F32::new(
          (baked_glyph.bbox.x) as f32 * ipw,
          (baked_glyph.bbox.y) as f32 * iph,
        ),
        uv_bottom_right: Vec2F32::new(
          (baked_glyph.bbox.x + baked_glyph.bbox.w) as f32 * ipw,
          (baked_glyph.bbox.y + baked_glyph.bbox.h) as f32 * iph,
        ),
      };

      font_glyphs_table.insert(baked_glyph.codepoint, new_glyph);
    });

    // copy glyph pixels into the atlas texture
    let mut atlas_pixels = vec![
      RGBAColor::new_with_alpha(0, 0, 0, 0);
      (atlas_width * atlas_height) as usize
    ];

    baked_glyphs.iter().for_each(|baked_glyph| {
      let bbox = baked_glyph.bbox;
      let mut src_idx = 0u32;
      (bbox.y .. (bbox.y + bbox.h)).for_each(|y| {
        (bbox.x .. (bbox.x + bbox.w)).for_each(|x| {
          let dst_idx = (y as u32 * atlas_width + x as u32) as usize;
          atlas_pixels[dst_idx] = baked_glyph.pixels[src_idx as usize];
          src_idx += 1;
        });
      });
    });

    let pixels_slice = unsafe {
      std::slice::from_raw_parts(
        atlas_pixels.as_ptr() as *const u8,
        atlas_pixels.len() * std::mem::size_of::<RGBAColor>(),
      )
    };

    fn_device_glyph_image_upload(atlas_width, atlas_height, pixels_slice)
      .and_then(|(glyphs_texture, draw_null_texture)| {
        // Move all data to the atlas, our job is done.
        let mut boxed_atlas = unsafe { Box::from_raw(self.atlas) };
        boxed_atlas.glyphs_texture = glyphs_texture;
        boxed_atlas.draw_null_texture = draw_null_texture;
        boxed_atlas.configs = std::mem::replace(&mut self.configs, vec![]);
        boxed_atlas.faces = std::mem::replace(&mut self.faces, vec![]);
        boxed_atlas.fonts = std::mem::replace(&mut self.fonts, vec![]);
        boxed_atlas.glyphs = std::mem::replace(&mut self.glyphs, vec![]);

        Some(boxed_atlas)
      })
      .ok_or("Failed to upload atlas to device!")
  }

  /// Add a TTF font from bytes.
  fn add_font_from_bytes(
    &mut self,
    font: &FontConfig,
    ttf_bytes: &[u8],
  ) -> Option<Font> {
    UniqueResource::<FreetypeFaceHandle>::from_handle(unsafe {
      let mut face: FT_Face = std::ptr::null_mut();
      FT_New_Memory_Face(
        *self.lib.handle(),
        ttf_bytes.as_ptr() as *const FT_Byte,
        ttf_bytes.len() as FT_Long,
        0,
        &mut face as *mut _,
      );

      face
    })
    .and_then(|face| {
      let face_metrics =
        FontMetrics::extract(*face.handle(), font.size, self.dpi);

      let font_handle = self.fonts.len() as u32;
      let face_handle = self.faces.len() as u32;

      font.glyph_range.iter().for_each(|glyphrange| {
        (glyphrange.start as u32 .. glyphrange.end as u32).for_each(
          |codepoint| {
            extract_glyph_spans(codepoint, *face.handle(), *self.lib.handle())
              .map(|(bearing_x, bearing_y, advance_x, glyph_spans)| {
                self.baked_glyphs.push(BakedGlyph::new(
                  codepoint,
                  font_handle,
                  bearing_x as f32,
                  bearing_y as f32,
                  font.calc_xadvance(advance_x),
                  &glyph_spans,
                ));
              });
          },
        );
      });

      // Extract the fallback glyph. This may already have been extracted if
      // it was in the configured glyph range.
      self
        .baked_glyphs
        .iter()
        .find(|baked_glyph| baked_glyph.codepoint == font.fallback_glyph as u32)
        .map_or_else(
          || {
            // fallback glyph not found, extract its data and return it for
            // insertion
            let fallback_glyph = extract_glyph_spans(
              font.fallback_glyph as u32,
              *face.handle(),
              *self.lib.handle(),
            )
            .map(|(bearing_x, bearing_y, advance, glyph_spans)| {
              BakedGlyph::new(
                font.fallback_glyph as u32,
                font_handle,
                bearing_x as f32,
                bearing_y as f32,
                font.calc_xadvance(advance),
                &glyph_spans,
              )
            })
            .unwrap_or_else(|| {
              BakedGlyph::new(
                font.fallback_glyph as u32,
                font_handle,
                0f32,
                0f32,
                font.calc_xadvance(face_metrics.max_advance_width as i32),
                &vec![],
              )
            });

            Some(fallback_glyph)
          },
          |_| {
            // fallback glyph already there , so nothing to do
            None
          },
        )
        .and_then(|fb_glyph| {
          self.baked_glyphs.push(fb_glyph);
          Some(())
        });

      self.faces.push(face_metrics);
      let this_font = Font {
        scale:     font.size,
        glyph_tbl: font_handle,
        face_tbl:  face_handle,
        atlas:     self.atlas,
      };
      self.fonts.push(this_font);
      self.glyphs.push(HashMap::new());
      self.configs.push(font.clone());

      Some(this_font)
    })
  }
}

pub struct FontAtlas {
  glyphs:            Vec<HashMap<u32, FontGlyph>>,
  fonts:             Vec<Font>,
  faces:             Vec<FontMetrics>,
  configs:           Vec<FontConfig>,
  glyphs_texture:    GenericHandle,
  draw_null_texture: DrawNullTexture,
}

impl FontAtlas {
  fn new() -> FontAtlas {
    FontAtlas {
      glyphs:            vec![],
      fonts:             vec![],
      faces:             vec![],
      configs:           vec![],
      glyphs_texture:    GenericHandle::Id(0),
      draw_null_texture: DrawNullTexture::default(),
    }
  }

  /// Query the properties of a font's glyph.
  pub fn query_font_glyph(
    &self,
    font: &Font,
    height: f32,
    codepoint: char,
  ) -> UserFontGlyph {
    let glyph_table = &self.glyphs[font.glyph_tbl as usize];
    glyph_table.get(&(codepoint as u32)).map_or(
      UserFontGlyph {
        uv:       [Vec2F32::same(0f32); 2],
        offset:   Vec2F32::same(0f32),
        width:    0f32,
        height:   0f32,
        xadvance: 0f32,
      },
      |glyph| {
        let scale = height / font.scale;
        UserFontGlyph {
          uv:       [glyph.uv_top_left, glyph.uv_bottom_right],
          offset:   Vec2F32::new(glyph.bearing_x, glyph.bearing_y) * scale,
          width:    glyph.bbox.w as f32 * scale,
          height:   glyph.bbox.h as f32 * scale,
          xadvance: glyph.xadvance * scale,
        }
      },
    )
  }

  /// Compute the length of a string using a certain font in the atlas.
  pub fn font_text_width(&self, font: &Font, height: f32, text: &str) -> f32 {
    let scale = height / font.scale;

    text.chars().fold(0f32, |curr_width, codepoint| {
      let glyph = self.query(font, codepoint);
      curr_width + glyph.xadvance * scale
    })
  }

  fn query(&self, font: &Font, codepoint: char) -> FontGlyph {
    let glyph_table = &self.glyphs[font.glyph_tbl as usize];
    glyph_table
      .get(&(codepoint as u32))
      .map_or(FontGlyph::default(), |glyph_entry| *glyph_entry)
  }

  pub fn clamp_text(
    &self,
    font: &Font,
    text: &str,
    max_width: f32,
  ) -> (i32, f32) {
    let mut glyph_count = 0;
    let mut width = 0f32;
    text.chars().all(|codepoint| {
      let glyph_info = self.query(font, codepoint);
      if (width + glyph_info.xadvance) > max_width {
        false
      } else {
        width += glyph_info.xadvance;
        glyph_count += 1;
        true
      }
    });

    (glyph_count, width)
  }

  /// Create a string by clamping some text to a specified maximum width.
  pub fn clamped_string(
    &self,
    font: &Font,
    text: &str,
    max_width: f32,
  ) -> String {
    let mut width = 0f32;

    text
      .chars()
      .take_while(|codepoint| {
        let glyph_info = self.query(font, *codepoint);
        if (width + glyph_info.xadvance) < max_width {
          width += glyph_info.xadvance;
          true
        } else {
          false
        }
      })
      .collect()
  }
}
