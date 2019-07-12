#![allow(dead_code)]

use crate::{
  hmi::{
    freetype2::*,
    rendered_glyphs_store::{RenderedGlyph, RenderedGlyphsStore},
  },
  math::{
    colors::RGBAColor,
    rectangle::RectangleI32,
    utility::{roundup_multiple_of, roundup_next_power_of_two},
    vec2::Vec2F32,
  },
  sys::{
    memory_mapped_file::MemoryMappedFile,
    unique_resource::{ResourceDeleter, UniqueResource},
  },
};
use std::{collections::HashMap, rc::Rc};

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
      let mut raster_params: FT_Raster_Params = ::std::mem::zeroed();
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

    // copy spans to scratch buffer

    let mut glyph_pixels =
      vec![RGBAColor::new(0, 0, 0); (img_width * img_height) as usize];

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

enum TTFSource {
  StaticBytes(&'static [u8]),
  MemMappedFile(MemoryMappedFile),
  OwnedBytes(Vec<u8>),
}

impl TTFSource {
  fn get_bytes<'a>(&'a self) -> &'a [u8] {
    match *self {
      TTFSource::StaticBytes(sb) => sb,
      TTFSource::MemMappedFile(ref mm) => mm.as_slice(),
      TTFSource::OwnedBytes(ref ob) => &ob,
    }
  }
}

#[derive(Debug)]
struct StoredGlyphRecord {
  handle:       FT_Glyph,
  glyph_ft_idx: u32,
  body_spans:   Rc<Vec<Span>>,
  bbox:         RectangleI32,
  bearing_y:    i32,
  render_data:  RenderedGlyph,
  renderable:   bool,
}

#[derive(Debug, Copy, Clone)]
pub struct FaceMetrics {
  pub size:                i32,
  pub height:              i32,
  pub ascender:            i32,
  pub descender:           i32,
  pub max_advance_width:   i32,
  pub max_advance_height:  i32,
  pub underline_pos:       i32,
  pub underline_thickness: i32,
}

#[derive(Debug)]
struct FaceSizeEntry {
  metrics:           FaceMetrics,
  rendered_glyphs:   HashMap<char, StoredGlyphRecord>,
  rendered_outlines: HashMap<char, StoredGlyphRecord>,
}

impl FaceSizeEntry {
  pub fn get_glyph_entry<'a>(
    &'a mut self,
    lib: FT_Library,
    face: FT_Face,
    dpi: u32,
    glyph: char,
    glyph_render_data_cache: &Rc<dyn RenderedGlyphsStore>,
  ) -> Option<&'a StoredGlyphRecord> {
    if !self.rendered_glyphs.contains_key(&glyph) {
      unsafe {
        FT_Set_Char_Size(
          face,
          (self.metrics.size * 64) as FT_F26Dot6,
          0,
          dpi,
          dpi,
        );
      }

      let glyph_idx = unsafe { FT_Get_Char_Index(face, glyph as FT_ULong) };
      if glyph_idx == 0 {
        println!("Failed to find index for glyph {}", glyph);
        return None;
      }

      let res = unsafe { FT_Load_Glyph(face, glyph_idx, FT_LOAD_DEFAULT) };

      if res != 0 {
        println!("Failed to load glyph for {}", glyph);
        return None;
      }

      let (glyph_handle, glyph_spans, bearing_y) = unsafe {
        let g = (*face).glyph;

        let bearing_y = ((*(*face).glyph).metrics.horiBearingY >> 6) as i32;

        if (*g).format != FT_GLYPH_FORMAT_OUTLINE {
          println!("Not an outline format");
          return None;
        }

        let mut glyph_cpy: FT_Glyph = ::std::ptr::null_mut();
        let res = FT_Get_Glyph(g, &mut glyph_cpy as *mut _);
        if res != 0 {
          println!("Failed to get glyph from face slot!");
          return None;
        }

        // render the glyph's spans
        let mut this_glyph_spans: Vec<Span> = Vec::new();
        Span::render_spans(lib, &mut (*g).outline, &mut this_glyph_spans);
        (glyph_cpy, this_glyph_spans, bearing_y)
      };

      if glyph_spans.is_empty() {
        // non-renderable (space, tab, etc ...)
        self.rendered_glyphs.insert(
          glyph,
          StoredGlyphRecord {
            handle:       glyph_handle,
            glyph_ft_idx: glyph_idx,
            body_spans:   Rc::new(glyph_spans),
            bbox:         RectangleI32::new(0, 0, 0, 0),
            bearing_y:    0,
            render_data:  RenderedGlyph::default(),
            renderable:   false,
          },
        );
      } else {
        let bbox = Span::bounding_box(&glyph_spans);
        let render_data =
          glyph_render_data_cache.store_glyph(&glyph_spans).unwrap();

        self.rendered_glyphs.insert(
          glyph,
          StoredGlyphRecord {
            handle: glyph_handle,
            glyph_ft_idx: glyph_idx,
            body_spans: Rc::new(glyph_spans),
            bbox,
            bearing_y,
            render_data,
            renderable: true,
          },
        );
      }
    }

    self.rendered_glyphs.get(&glyph)
  }
}

struct FaceRecord {
  ttf:     TTFSource,
  entries: Vec<FaceSizeEntry>,
  face:    FT_Face,
}

impl FaceRecord {
  pub fn set_glyph_size(&self, size: i32, dpi: u32) {
    unsafe {
      FT_Set_Char_Size(self.face, (size * 64) as FT_F26Dot6, 0, dpi, dpi);
    }
  }

  fn get_face_size_entry<'a>(
    &'a mut self,
    size: i32,
    dpi: u32,
  ) -> Option<&'a mut FaceSizeEntry> {
    for i in 0 .. self.entries.len() {
      let curr_size = self.entries[i].metrics.size;
      if curr_size == size {
        return Some(&mut self.entries[i]);
      }
    }

    self.set_glyph_size(size, dpi);

    let pixel_size = size * dpi as i32 / 72;
    let units_per_em = unsafe { (*self.face).units_per_EM as i32 };
    // distance between 2 baselines
    let height =
      unsafe { (*self.face).height as i32 * pixel_size / units_per_em };

    let ascender =
      unsafe { (*self.face).ascender.abs() as i32 * pixel_size / units_per_em };

    let descender = unsafe {
      (*self.face).descender.abs() as i32 * pixel_size / units_per_em
    };

    let max_advance_width = unsafe {
      (*self.face).max_advance_width as i32 * pixel_size / units_per_em
    };

    let max_advance_height = unsafe {
      (*self.face).max_advance_height as i32 * pixel_size / units_per_em
    };
    let underline_pos = unsafe {
      (*self.face).underline_position as i32 * pixel_size / units_per_em
    };
    let underline_thickness = unsafe {
      (*self.face).underline_thickness as i32 * pixel_size / units_per_em
    };

    let metrics = FaceMetrics {
      size,
      height,
      ascender,
      descender,
      max_advance_width,
      max_advance_height,
      underline_pos,
      underline_thickness,
    };

    println!("Face metrics {:?}", metrics);

    self.entries.push(FaceSizeEntry {
      metrics,
      rendered_glyphs: HashMap::new(),
      rendered_outlines: HashMap::new(),
    });

    self.entries.last_mut()
  }
}

#[derive(Copy, Clone, Debug)]
pub struct FontId {
  name_hash: u64,
  size:      i32,
}

impl FontId {
  fn new(name: &str, size: i32) -> FontId {
    use std::{collections::hash_map::DefaultHasher, hash::Hasher};

    let mut hasher = DefaultHasher::new();
    hasher.write(name.as_bytes());

    FontId {
      name_hash: hasher.finish(),
      size,
    }
  }

  pub fn get_text_width(s: &str) -> f32 {
    0f32
  }

  pub fn get_glyph_info(codepoint: char, next_codepoint: char) -> f32 {
    0f32
  }
}

impl ::std::hash::Hash for FontId {
  fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
    self.name_hash.hash(state);
  }
}

impl ::std::cmp::PartialEq for FontId {
  fn eq(&self, other: &FontId) -> bool {
    self.name_hash == other.name_hash
  }
}

impl ::std::cmp::Eq for FontId {}

#[derive(Clone, Debug)]
pub struct FontConfig {
  pub pixel_snap:     bool,
  pub oversample_v:   u8,
  pub oversample_h:   u8,
  pub size:           f32,
  pub spacing:        Vec2F32,
  pub glyph_range:    Vec<std::ops::Range<char>>,
  pub fallback_glyph: char,
}

impl FontConfig {
  pub fn new(pixel_height: f32) -> FontConfig {
    FontConfig {
      pixel_snap:     false,
      oversample_v:   1,
      oversample_h:   3,
      size:           pixel_height,
      spacing:        Vec2F32::new(0f32, 0f32),
      glyph_range:    FontConfig::default_glyph_ranges(),
      fallback_glyph: '?',
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
        (*face).ascender.abs() as i32 * pixel_size / units_per_em
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
  scale:     f32,
  glyph_tbl: u32,
  face_tbl:  u32,
}

pub struct FontGlyph {
  codepoint:       u32,
  xadvance:        f32,
  bearing_y:       f32,
  bbox:            RectangleI32,
  uv_top_left:     Vec2F32,
  uv_bottom_right: Vec2F32,
  /* x0:        f32,
   * y0:        f32,
   * x1:        f32,
   * y1:        f32,
   * w:         f32,
   * h:         f32, */
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
  bearing_y: i32,
  // index in the font table
  font:      u32,
  codepoint: u32,
  bbox:      RectangleI32,
  pixels:    Vec<RGBAColor>,
}

fn extract_glyph_spans(
  codepoint: u32,
  face: FT_Face,
  lib: FT_Library,
) -> Option<(i32, Vec<Span>)> {
  let ft_glyph_index =
    unsafe { FT_Get_Char_Index(face, codepoint as FT_ULong) };

  if ft_glyph_index == 0 {
    return None;
  }

  let load_result =
    unsafe { FT_Load_Glyph(face, ft_glyph_index, FT_LOAD_DEFAULT) };

  if load_result != 0 {
    return None;
  }

  let g = unsafe { (*face).glyph };
  let glyph_format = unsafe { (*g).format };
  if glyph_format != FT_GLYPH_FORMAT_OUTLINE {
    return None;
  }

  let bearing_y =
    unsafe { ((*(*face).glyph).metrics.horiBearingY >> 6) as i32 };

  let glyph_cpy = unsafe {
    let mut glyph_cpy: FT_Glyph = std::ptr::null_mut();
    FT_Get_Glyph(g, &mut glyph_cpy as *mut _);
    glyph_cpy
  };

  if glyph_cpy.is_null() {
    return None;
  }

  let mut glyph_spans = Vec::<Span>::new();
  unsafe {
    Span::render_spans(lib, &mut (*g).outline, &mut glyph_spans);
  }
  Some((bearing_y, glyph_spans))
}

/// Pack the baked glyphs into a rectangular texture
fn pack_rects(rects: &mut [BakedGlyph]) -> (u32, u32, f32) {
  let (area, max_width) = rects.iter().fold((0, 0), |acc, r| {
    (acc.0 + r.bbox.w * r.bbox.h, acc.1.max(r.bbox.w))
  });

  let non_renderables = rects
    .iter()
    .filter(|glyph| glyph.bbox.w == 0 || glyph.bbox.h == 0)
    .count();

  rects.sort_by(|glyph_a, glyph_b| glyph_b.bbox.h.cmp(&glyph_a.bbox.h));

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

pub struct FontAtlas {
  glyphs:        HashMap<u32, Vec<FontGlyph>>,
  baked_glyphs:  Vec<BakedGlyph>,
  glyphs_pixels: Vec<RGBAColor>,
  fonts:         Vec<Font>,
  faces:         Vec<FontMetrics>,
  configs:       Vec<FontConfig>,
  stroker:       UniqueResource<FreetypeStrokerHandle>,
  lib:           UniqueResource<FreetypeLibraryHandle>,
}

impl FontAtlas {
  const DPI: u32 = 300;

  pub fn build(&mut self) -> Option<(u32, u32, Vec<RGBAColor>)> {
    assert!(!self.fonts.is_empty(), "You forgot to add any fonts!");
    assert!(
      !self.baked_glyphs.is_empty(),
      "You forgot to add any fonts!"
    );

    if self.baked_glyphs.is_empty() {
      return None;
    }

    let (atlas_width, atlas_height, _) = pack_rects(&mut self.baked_glyphs);
    if atlas_width == 0 || atlas_height == 0 {
      return None;
    }

    let (atlas_width, atlas_height) = (
      roundup_multiple_of(atlas_width, 4),
      roundup_multiple_of(atlas_height, 4),
      /* roundup_next_power_of_two(atlas_width),
       * roundup_next_power_of_two(atlas_height), */
    );

    let mut glyphs_table = HashMap::new();

    self.baked_glyphs.iter().for_each(|baked_glyph| {
      let font_glyphs_table = glyphs_table
        .entry(baked_glyph.font)
        .or_insert(Vec::<FontGlyph>::new());

      font_glyphs_table.push(FontGlyph {
        codepoint:       baked_glyph.codepoint,
        xadvance:        0f32,
        bearing_y:       baked_glyph.bearing_y as f32,
        bbox:            RectangleI32::new(
          0,
          0,
          baked_glyph.bbox.w,
          baked_glyph.bbox.h,
        ),
        uv_top_left:     Vec2F32::new(
          baked_glyph.bbox.x as f32 / atlas_width as f32,
          baked_glyph.bbox.y as f32 / atlas_height as f32,
        ),
        uv_bottom_right: Vec2F32::new(
          (baked_glyph.bbox.x + baked_glyph.bbox.w) as f32 / atlas_width as f32,
          (baked_glyph.bbox.y + baked_glyph.bbox.h) as f32
            / atlas_height as f32,
        ),
      });
    });

    self.glyphs = glyphs_table;

    // copy glyph pixels into the atlas texture
    let mut atlas_pixels =
      vec![RGBAColor::new(0, 0, 0); (atlas_width * atlas_height) as usize];

    self.baked_glyphs.iter().for_each(|baked_glyph| {
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

    Some((atlas_width, atlas_height, atlas_pixels))
  }

  pub fn new() -> Option<FontAtlas> {
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
        Some(FontAtlas {
          glyphs: HashMap::new(),
          baked_glyphs: Vec::new(),
          glyphs_pixels: Vec::new(),
          fonts: Vec::new(),
          faces: Vec::new(),
          configs: Vec::new(),
          stroker,
          lib: ftlib,
        })
      })
    })
  }

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
        FontMetrics::extract(*face.handle(), font.size, Self::DPI);
      let font_handle = self.fonts.len() as u32;
      let face_handle = self.faces.len() as u32;

      font.glyph_range.iter().for_each(|glyphrange| {
        (glyphrange.start as u32 .. glyphrange.end as u32).for_each(
          |codepoint| {
            extract_glyph_spans(codepoint, *face.handle(), *self.lib.handle())
              .map(|(bearing_y, glyph_spans)| {
                let baked_glyph = if glyph_spans.is_empty() {
                  // non renderable (space, tab, newline, etc ...)
                  BakedGlyph {
                    codepoint,
                    bearing_y,
                    font: font_handle,
                    bbox: RectangleI32::new(0, 0, 0, 0),
                    pixels: vec![],
                  }
                } else {
                  let (glyph_bbox, glyph_pixels) =
                    Span::convert_to_pixels(&glyph_spans);

                  BakedGlyph {
                    codepoint,
                    bearing_y,
                    font: font_handle,
                    bbox: glyph_bbox,
                    pixels: glyph_pixels,
                  }
                };

                self.baked_glyphs.push(baked_glyph);
              });
          },
        );
      });

      self.faces.push(face_metrics);
      let this_font = Font {
        scale:     font.size,
        glyph_tbl: font_handle,
        face_tbl:  face_handle,
      };
      self.fonts.push(this_font);
      self.configs.push(font.clone());

      Some(this_font)
    })
  }
}

pub struct TextEngineOptions {
  pub dpi:                   u32,
  pub surface_target_width:  i32,
  pub surface_target_height: i32,
  pub rendered_glyphs_store: Rc<dyn RenderedGlyphsStore>,
}

pub struct TextEngine {
  rendered_glyphs_store: Rc<dyn RenderedGlyphsStore>,
  dpi:                   u32,
  ftlib:                 FT_Library,
  faces_cache:           HashMap<FontId, FaceRecord>,
  stroker:               FT_Stroker,
}

impl TextEngine {
  pub fn add_font_from_file(
    &mut self,
    path: &std::path::Path,
    size: i32,
  ) -> Option<FontId> {
    let font_name = path.file_name().unwrap().to_str().unwrap();
    let font_id = FontId::new(font_name, size);

    if !self.faces_cache.contains_key(&font_id) {
      let face_file = MemoryMappedFile::new(&path).unwrap();

      let (face, res) = unsafe {
        let mut face: FT_Face = std::ptr::null_mut();
        let res = FT_New_Memory_Face(
          self.ftlib,
          face_file.as_slice().as_ptr() as *const FT_Byte,
          face_file.len() as FT_Long,
          0,
          &mut face as *mut _,
        );

        (face, res)
      };

      if res != 0 {
        println!("Failed to load font {}", path.display());
        return None;
      }

      let f_record = FaceRecord {
        face,
        ttf: TTFSource::MemMappedFile(face_file),
        entries: Vec::new(),
      };

      self.faces_cache.insert(font_id, f_record);
    }

    Some(font_id)
  }

  pub fn add_font_from_static_bytes(
    &mut self,
    font_name: &str,
    font_bytes: &'static [u8],
    size: i32,
  ) -> Option<FontId> {
    let font_id = FontId::new(font_name, size);

    if !self.faces_cache.contains_key(&font_id) {
      let (face, res) = unsafe {
        let mut face: FT_Face = std::ptr::null_mut();
        let res = FT_New_Memory_Face(
          self.ftlib,
          font_bytes.as_ptr() as *const FT_Byte,
          font_bytes.len() as FT_Long,
          0,
          &mut face as *mut _,
        );

        (face, res)
      };

      if res != 0 {
        println!("Failed to load font {}", font_name);
        return None;
      }

      let f_record = FaceRecord {
        face,
        ttf: TTFSource::StaticBytes(font_bytes),
        entries: Vec::new(),
      };

      self.faces_cache.insert(font_id, f_record);
    }

    Some(font_id)
  }

  fn get_face<'a>(&'a mut self, font: FontId) -> Option<&'a mut FaceRecord> {
    self.faces_cache.get_mut(&font)
  }

  pub fn new(params: &TextEngineOptions) -> TextEngine {
    let ftlib = {
      let mut ftlib: FT_Library = std::ptr::null_mut();
      let res = unsafe { FT_Init_FreeType(&mut ftlib as *mut _) };
      if res == 0 {
        Some(ftlib)
      } else {
        println!("Failed to initialize Freetype2 library!");
        None
      }
    }
    .unwrap();

    let stroker = {
      let mut stroker: FT_Stroker = ::std::ptr::null_mut();
      let result = unsafe { FT_Stroker_New(ftlib, &mut stroker as *mut _) };

      if result != 0 {
        println!("Failed to create stroker!");
        None
      } else {
        Some(stroker)
      }
    }
    .unwrap();

    TextEngine {
      rendered_glyphs_store: Rc::clone(&params.rendered_glyphs_store),
      dpi: params.dpi,
      ftlib,
      faces_cache: HashMap::new(),
      stroker,
    }
  }
}

impl ::std::ops::Drop for TextEngine {
  fn drop(&mut self) {
    self.faces_cache.values_mut().for_each(|face_rec| {
      face_rec.entries.iter().for_each(|face_entry| {
        face_entry
          .rendered_glyphs
          .values()
          .for_each(|cached_glyph| unsafe {
            FT_Done_Glyph(cached_glyph.handle);
          });

        face_entry
          .rendered_outlines
          .values()
          .for_each(|cached_glyph| unsafe {
            FT_Done_Glyph(cached_glyph.handle);
          });
      });

      unsafe {
        FT_Done_Face(face_rec.face);
      }
    });

    unsafe {
      FT_Done_Library(self.ftlib);
    }
  }
}
