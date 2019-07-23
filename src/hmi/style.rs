use crate::{
  hmi::{
    cursor::Cursor,
    image::Image,
    text::{TextAlign, TextAlignment},
    text_engine::Font,
  },
  math::{colors::RGBAColor, vec2::Vec2F32},
};
use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Copy, Clone, Debug, FromPrimitive, ToPrimitive)]
pub enum SymbolType {
  X,
  None_,
  Underscore,
  CircleSolid,
  CircleOutline,
  RectSolid,
  RectOutline,
  TriangleUp,
  TriangleDown,
  TriangleLeft,
  TriangleRight,
  Plus,
  Minus,
  Max,
}

#[derive(Copy, Clone, Debug)]
pub enum StyleItem {
  Img(Image),
  Color(RGBAColor),
}

impl StyleItem {
  fn hide() -> StyleItem {
    StyleItem::Color(RGBAColor::new_with_alpha(0, 0, 0, 0))
  }
}

#[derive(Copy, Clone, Debug)]
pub struct StyleText {
  pub color:   RGBAColor,
  pub padding: Vec2F32,
}

#[derive(Copy, Clone, Debug)]
pub struct StyleButton {
  // background
  pub normal:       StyleItem,
  pub hover:        StyleItem,
  pub active:       StyleItem,
  pub border_color: RGBAColor,

  // text
  pub text_background: RGBAColor,
  pub text_normal:     RGBAColor,
  pub text_hover:      RGBAColor,
  pub text_active:     RGBAColor,
  pub text_alignment:  u32,

  // properties
  pub border:        f32,
  pub rounding:      f32,
  pub padding:       Vec2F32,
  pub image_padding: Vec2F32,
  pub touch_padding: Vec2F32,
}

#[derive(Copy, Clone, Debug)]
pub struct StyleToggle {
  // background
  pub normal:       StyleItem,
  pub hover:        StyleItem,
  pub active:       StyleItem,
  pub border_color: RGBAColor,

  // cursor
  pub cursor_normal: StyleItem,
  pub cursor_hover:  StyleItem,

  // text
  pub text_background: RGBAColor,
  pub text_normal:     RGBAColor,
  pub text_hover:      RGBAColor,
  pub text_active:     RGBAColor,
  pub text_alignment:  u32,

  // properties
  pub border:        f32,
  pub spacing:       f32,
  pub padding:       Vec2F32,
  pub touch_padding: Vec2F32,
}

#[derive(Copy, Clone, Debug)]
pub struct StyleSelectable {
  // background (inactive)
  pub normal:  StyleItem,
  pub hover:   StyleItem,
  pub pressed: StyleItem,

  // background (active)
  pub normal_active:  StyleItem,
  pub hover_active:   StyleItem,
  pub pressed_active: StyleItem,

  // text (inactive)
  pub text_normal:  RGBAColor,
  pub text_hover:   RGBAColor,
  pub text_pressed: RGBAColor,

  // text (active)
  pub text_normal_active:  RGBAColor,
  pub text_hover_active:   RGBAColor,
  pub text_pressed_active: RGBAColor,
  pub text_background:     RGBAColor,
  pub text_alignment:      u32,

  // properties
  pub rounding:      f32,
  pub padding:       Vec2F32,
  pub touch_padding: Vec2F32,
  pub image_padding: Vec2F32,
}

#[derive(Copy, Clone, Debug)]
pub struct StyleSlider {
  // background
  pub normal:       StyleItem,
  pub hover:        StyleItem,
  pub active:       StyleItem,
  pub border_color: RGBAColor,

  // background bar *
  pub bar_normal: RGBAColor,
  pub bar_hover:  RGBAColor,
  pub bar_active: RGBAColor,
  pub bar_filled: RGBAColor,

  // cursor *
  pub cursor_normal: StyleItem,
  pub cursor_hover:  StyleItem,
  pub cursor_active: StyleItem,

  // properties *
  pub border:      f32,
  pub rounding:    f32,
  pub bar_height:  f32,
  pub padding:     Vec2F32,
  pub spacing:     Vec2F32,
  pub cursor_size: Vec2F32,

  // optional buttons *
  pub show_buttons: bool,
  pub inc_button:   StyleButton,
  pub dec_button:   StyleButton,
  pub inc_symbol:   SymbolType,
  pub dec_symbol:   SymbolType,
}

#[derive(Copy, Clone, Debug)]
pub struct StyleProgress {
  // background
  pub normal:       StyleItem,
  pub hover:        StyleItem,
  pub active:       StyleItem,
  pub border_color: RGBAColor,

  // cursor
  pub cursor_normal:       StyleItem,
  pub cursor_hover:        StyleItem,
  pub cursor_active:       StyleItem,
  pub cursor_border_color: RGBAColor,

  // properties
  pub rounding:        f32,
  pub border:          f32,
  pub cursor_border:   f32,
  pub cursor_rounding: f32,
  pub padding:         Vec2F32,
}

#[derive(Copy, Clone, Debug)]
pub struct StyleScrollbar {
  // background
  pub normal:       StyleItem,
  pub hover:        StyleItem,
  pub active:       StyleItem,
  pub border_color: RGBAColor,

  // cursor
  pub cursor_normal:       StyleItem,
  pub cursor_hover:        StyleItem,
  pub cursor_active:       StyleItem,
  pub cursor_border_color: RGBAColor,

  // properties
  pub border:          f32,
  pub rounding:        f32,
  pub border_cursor:   f32,
  pub rounding_cursor: f32,
  pub padding:         Vec2F32,

  // optional buttons *
  pub show_buttons: bool,
  pub inc_button:   StyleButton,
  pub dec_button:   StyleButton,
  pub inc_symbol:   SymbolType,
  pub dec_symbol:   SymbolType,
}

#[derive(Copy, Clone, Debug)]
pub struct StyleEdit {
  // background
  pub normal:       StyleItem,
  pub hover:        StyleItem,
  pub active:       StyleItem,
  pub border_color: RGBAColor,
  pub scrollbar:    StyleScrollbar,

  // cursor
  pub cursor_normal:      RGBAColor,
  pub cursor_hover:       RGBAColor,
  pub cursor_text_normal: RGBAColor,
  pub cursor_text_hover:  RGBAColor,

  // text (unselected)
  pub text_normal: RGBAColor,
  pub text_hover:  RGBAColor,
  pub text_active: RGBAColor,

  // text (selected)
  pub selected_normal:      RGBAColor,
  pub selected_hover:       RGBAColor,
  pub selected_text_normal: RGBAColor,
  pub selected_text_hover:  RGBAColor,

  // properties
  pub border:         f32,
  pub rounding:       f32,
  pub cursor_size:    f32,
  pub scrollbar_size: Vec2F32,
  pub padding:        Vec2F32,
  pub row_padding:    f32,
}

#[derive(Copy, Clone, Debug)]
pub struct StyleProperty {
  // background
  pub normal:       StyleItem,
  pub hover:        StyleItem,
  pub active:       StyleItem,
  pub border_color: RGBAColor,

  // text
  pub label_normal: RGBAColor,
  pub label_hover:  RGBAColor,
  pub label_active: RGBAColor,

  // symbols
  pub sym_left:  SymbolType,
  pub sym_right: SymbolType,

  // properties
  pub border:   f32,
  pub rounding: f32,
  pub padding:  Vec2F32,

  pub edit:       StyleEdit,
  pub inc_button: StyleButton,
  pub dec_button: StyleButton,
}

#[derive(Copy, Clone, Debug)]
pub struct StyleChart {
  // colors
  pub background:     StyleItem,
  pub border_color:   RGBAColor,
  pub selected_color: RGBAColor,
  pub color:          RGBAColor,

  // properties
  pub border:   f32,
  pub rounding: f32,
  pub padding:  Vec2F32,
}

#[derive(Copy, Clone, Debug)]
pub struct StyleCombo {
  // background
  pub normal:       StyleItem,
  pub hover:        StyleItem,
  pub active:       StyleItem,
  pub border_color: RGBAColor,

  // label
  pub label_normal: RGBAColor,
  pub label_hover:  RGBAColor,
  pub label_active: RGBAColor,

  // symbol
  pub symbol_normal: RGBAColor,
  pub symbol_hover:  RGBAColor,
  pub symbol_active: RGBAColor,

  // button
  pub button:     StyleButton,
  pub sym_normal: SymbolType,
  pub sym_hover:  SymbolType,
  pub sym_active: SymbolType,

  // properties
  pub border:          f32,
  pub rounding:        f32,
  pub content_padding: Vec2F32,
  pub button_padding:  Vec2F32,
  pub spacing:         Vec2F32,
}

#[derive(Copy, Clone, Debug)]
pub struct StyleTab {
  // background
  pub background:   StyleItem,
  pub border_color: RGBAColor,
  pub text:         RGBAColor,

  // button
  pub tab_maximize_button:  StyleButton,
  pub tab_minimize_button:  StyleButton,
  pub node_maximize_button: StyleButton,
  pub node_minimize_button: StyleButton,
  pub sym_minimize:         SymbolType,
  pub sym_maximize:         SymbolType,

  // properties
  pub border:   f32,
  pub rounding: f32,
  pub indent:   f32,
  pub padding:  Vec2F32,
  pub spacing:  Vec2F32,
}

#[derive(Copy, Clone, Debug, FromPrimitive, ToPrimitive)]
pub enum StyleHeaderAlign {
  Left,
  Right,
}

#[derive(Copy, Clone, Debug)]
pub struct StyleWindowHeader {
  // background
  pub normal: StyleItem,
  pub hover:  StyleItem,
  pub active: StyleItem,

  // button
  pub close_button:    StyleButton,
  pub minimize_button: StyleButton,
  pub close_symbol:    SymbolType,
  pub minimize_symbol: SymbolType,
  pub maximize_symbol: SymbolType,

  // title
  pub label_normal: RGBAColor,
  pub label_hover:  RGBAColor,
  pub label_active: RGBAColor,

  // properties
  pub align:         StyleHeaderAlign,
  pub padding:       Vec2F32,
  pub label_padding: Vec2F32,
  pub spacing:       Vec2F32,
}

#[derive(Copy, Clone, Debug)]
pub struct StyleWindow {
  pub header:           StyleWindowHeader,
  pub fixed_background: StyleItem,
  pub background:       RGBAColor,

  pub border_color:            RGBAColor,
  pub popup_border_color:      RGBAColor,
  pub combo_border_color:      RGBAColor,
  pub contextual_border_color: RGBAColor,
  pub menu_border_color:       RGBAColor,
  pub group_border_color:      RGBAColor,
  pub tooltip_border_color:    RGBAColor,
  pub scaler:                  StyleItem,

  pub border:                 f32,
  pub combo_border:           f32,
  pub contextual_border:      f32,
  pub menu_border:            f32,
  pub group_border:           f32,
  pub tooltip_border:         f32,
  pub popup_border:           f32,
  pub min_row_height_padding: f32,

  pub rounding:       f32,
  pub spacing:        Vec2F32,
  pub scrollbar_size: Vec2F32,
  pub min_size:       Vec2F32,

  pub padding:            Vec2F32,
  pub group_padding:      Vec2F32,
  pub popup_padding:      Vec2F32,
  pub combo_padding:      Vec2F32,
  pub contextual_padding: Vec2F32,
  pub menu_padding:       Vec2F32,
  pub tooltip_padding:    Vec2F32,
}

#[derive(Copy, Clone, Debug)]
pub enum StyleColors {
  ColorText,
  ColorWindow,
  ColorHeader,
  ColorBorder,
  ColorButton,
  ColorButtonHover,
  ColorButtonActive,
  ColorToggle,
  ColorToggleHover,
  ColorToggleCursor,
  ColorSelect,
  ColorSelectActive,
  ColorSlider,
  ColorSliderCursor,
  ColorSliderCursorHover,
  ColorSliderCursorActive,
  ColorProperty,
  ColorEdit,
  ColorEditCursor,
  ColorCombo,
  ColorChart,
  ColorChartColor,
  ColorChartColorHighlight,
  ColorScrollbar,
  ColorScrollbarCursor,
  ColorScrollbarCursorHover,
  ColorScrollbarCursorActive,
  ColorTabHeader,
  ColorCount,
}

#[derive(Copy, Clone, Debug)]
pub enum StyleCursor {
  CursorArrow,
  CursorText,
  CursorMove,
  CursorResizeVertical,
  CursorResizeHorizontal,
  CursorResizeTopLeftDownRight,
  CursorResizeTopRightDownLeft,
  CursorCount,
}

#[derive(Copy, Clone, Debug)]
pub struct Style {
  pub font:              Font,
  pub cursors:           [Cursor; Self::CURSOR_COUNT as usize],
  pub cursor_active:     usize,
  pub cursor_last:       usize,
  pub cursor_visible:    bool,
  pub text:              StyleText,
  pub button:            StyleButton,
  pub contextual_button: StyleButton,
  pub menu_button:       StyleButton,
  pub option:            StyleToggle,
  pub checkbox:          StyleToggle,
  pub selectable:        StyleSelectable,
  pub slider:            StyleSlider,
  pub progress:          StyleProgress,
  pub property:          StyleProperty,
  pub edit:              StyleEdit,
  pub chart:             StyleChart,
  pub scrollh:           StyleScrollbar,
  pub scrollv:           StyleScrollbar,
  pub tab:               StyleTab,
  pub combo:             StyleCombo,
  pub window:            StyleWindow,
}

impl Style {
  pub const CURSOR_COUNT: i32 = 7;

  // fn default_color_style() -> &[RGBAColor] {

  // }
  // DEFAULT_COLOR_STYLE = [RGBAColor::new(0, 0, 0); 1];

  pub fn from_table(table: &[RGBAColor]) {
    // default button
    let text = StyleText {
      color:   table[StyleColors::ColorText as usize],
      padding: Vec2F32::same(0f32),
    };

    // default text
    let button = StyleButton {
      normal:          StyleItem::Color(
        table[StyleColors::ColorButton as usize],
      ),
      hover:           StyleItem::Color(
        table[StyleColors::ColorButtonHover as usize],
      ),
      active:          StyleItem::Color(
        table[StyleColors::ColorButtonActive as usize],
      ),
      border_color:    table[StyleColors::ColorBorder as usize],
      text_background: table[StyleColors::ColorButton as usize],
      text_normal:     table[StyleColors::ColorText as usize],
      text_hover:      table[StyleColors::ColorText as usize],
      text_active:     table[StyleColors::ColorText as usize],
      padding:         Vec2F32::same(2f32),
      image_padding:   Vec2F32::same(0f32),
      touch_padding:   Vec2F32::same(0f32),
      text_alignment:  TextAlignment::Centered,
      border:          1f32,
      rounding:        4f32,
    };

    let contextual_button = StyleButton {
      normal:          StyleItem::Color(
        table[StyleColors::ColorWindow as usize],
      ),
      hover:           StyleItem::Color(
        table[StyleColors::ColorButtonHover as usize],
      ),
      active:          StyleItem::Color(
        table[StyleColors::ColorButtonActive as usize],
      ),
      border_color:    table[StyleColors::ColorWindow as usize],
      text_background: table[StyleColors::ColorWindow as usize],
      text_normal:     table[StyleColors::ColorText as usize],
      text_hover:      table[StyleColors::ColorText as usize],
      text_active:     table[StyleColors::ColorText as usize],
      padding:         Vec2F32::same(2f32),
      image_padding:   Vec2F32::same(0f32),
      touch_padding:   Vec2F32::same(0f32),
      text_alignment:  TextAlignment::Centered,
      border:          0f32,
      rounding:        0f32,
    };

    let menu_button = StyleButton {
      normal:          StyleItem::Color(
        table[StyleColors::ColorWindow as usize],
      ),
      hover:           StyleItem::Color(
        table[StyleColors::ColorWindow as usize],
      ),
      active:          StyleItem::Color(
        table[StyleColors::ColorWindow as usize],
      ),
      border_color:    table[StyleColors::ColorWindow as usize],
      text_background: table[StyleColors::ColorWindow as usize],
      text_normal:     table[StyleColors::ColorText as usize],
      text_hover:      table[StyleColors::ColorText as usize],
      text_active:     table[StyleColors::ColorText as usize],
      padding:         Vec2F32::same(2f32),
      image_padding:   Vec2F32::same(0f32),
      touch_padding:   Vec2F32::same(0f32),
      text_alignment:  TextAlignment::Centered,
      border:          0f32,
      rounding:        1f32,
    };

    // checkbox toggle
    let toggle = StyleToggle {
      normal:          StyleItem::Color(
        table[StyleColors::ColorToggle as usize],
      ),
      hover:           StyleItem::Color(
        table[StyleColors::ColorToggleHover as usize],
      ),
      active:          StyleItem::Color(
        table[StyleColors::ColorToggleHover as usize],
      ),
      cursor_normal:   StyleItem::Color(
        table[StyleColors::ColorToggleCursor as usize],
      ),
      cursor_hover:    StyleItem::Color(
        table[StyleColors::ColorToggleCursor as usize],
      ),
      text_background: table[StyleColors::ColorWindow as usize],
      text_normal:     table[StyleColors::ColorText as usize],
      text_hover:      table[StyleColors::ColorText as usize],
      text_alignment:  TextAlignment::Centered,
      text_active:     table[StyleColors::ColorText as usize],
      padding:         Vec2F32::same(2f32),
      touch_padding:   Vec2F32::same(0f32),
      border_color:    RGBAColor::new(0, 0, 0),
      border:          0f32,
      spacing:         4f32,
    };

    let option = StyleToggle {
      normal:          StyleItem::Color(
        table[StyleColors::ColorToggle as usize],
      ),
      hover:           StyleItem::Color(
        table[StyleColors::ColorToggleHover as usize],
      ),
      active:          StyleItem::Color(
        table[StyleColors::ColorToggleHover as usize],
      ),
      cursor_normal:   StyleItem::Color(
        table[StyleColors::ColorToggleCursor as usize],
      ),
      cursor_hover:    StyleItem::Color(
        table[StyleColors::ColorToggleCursor as usize],
      ),
      text_background: table[StyleColors::ColorWindow as usize],
      text_normal:     table[StyleColors::ColorText as usize],
      text_hover:      table[StyleColors::ColorText as usize],
      text_alignment:  TextAlignment::Centered,
      text_active:     table[StyleColors::ColorText as usize],
      padding:         Vec2F32::same(3f32),
      touch_padding:   Vec2F32::same(0f32),
      border_color:    RGBAColor::new(0, 0, 0),
      border:          0f32,
      spacing:         4f32,
    };

    let select = StyleSelectable {
      normal:              StyleItem::Color(
        table[StyleColors::ColorSelect as usize],
      ),
      hover:               StyleItem::Color(
        table[StyleColors::ColorSelect as usize],
      ),
      pressed:             StyleItem::Color(
        table[StyleColors::ColorSelect as usize],
      ),
      normal_active:       StyleItem::Color(
        table[StyleColors::ColorSelectActive as usize],
      ),
      hover_active:        StyleItem::Color(
        table[StyleColors::ColorSelectActive as usize],
      ),
      pressed_active:      StyleItem::Color(
        table[StyleColors::ColorSelectActive as usize],
      ),
      text_alignment:      TextAlignment::Centered,
      text_background:     RGBAColor::new(0, 0, 0),
      text_normal:         table[StyleColors::ColorText as usize],
      text_hover:          table[StyleColors::ColorText as usize],
      text_pressed:        table[StyleColors::ColorText as usize],
      text_normal_active:  table[StyleColors::ColorText as usize],
      text_hover_active:   table[StyleColors::ColorText as usize],
      text_pressed_active: table[StyleColors::ColorText as usize],
      padding:             Vec2F32::same(2f32),
      image_padding:       Vec2F32::same(2f32),
      touch_padding:       Vec2F32::same(0f32),
      rounding:            0f32,
    };

    let slider_btn = StyleButton {
      normal:          StyleItem::Color(RGBAColor::new(40, 40, 40)),
      hover:           StyleItem::Color(RGBAColor::new(42, 42, 42)),
      active:          StyleItem::Color(RGBAColor::new(44, 44, 44)),
      border_color:    RGBAColor::new(65, 65, 65),
      text_background: RGBAColor::new(40, 40, 40),
      text_normal:     RGBAColor::new(175, 175, 175),
      text_hover:      RGBAColor::new(175, 175, 175),
      text_active:     RGBAColor::new(175, 175, 175),
      padding:         Vec2F32::same(8f32),
      image_padding:   Vec2F32::same(0f32),
      touch_padding:   Vec2F32::same(0f32),
      text_alignment:  TextAlignment::Centered,
      border:          1f32,
      rounding:        0f32,
    };

    let slider = StyleSlider {
      normal:        StyleItem::hide(),
      hover:         StyleItem::hide(),
      active:        StyleItem::hide(),
      bar_normal:    table[StyleColors::ColorSlider as usize],
      bar_hover:     table[StyleColors::ColorSlider as usize],
      bar_active:    table[StyleColors::ColorSlider as usize],
      bar_filled:    table[StyleColors::ColorSliderCursor as usize],
      cursor_normal: StyleItem::Color(
        table[StyleColors::ColorSliderCursor as usize],
      ),
      cursor_hover:  StyleItem::Color(
        table[StyleColors::ColorSliderCursorHover as usize],
      ),
      cursor_active: StyleItem::Color(
        table[StyleColors::ColorSliderCursorActive as usize],
      ),
      inc_symbol:    SymbolType::TriangleRight,
      dec_symbol:    SymbolType::TriangleLeft,
      inc_button:    slider_btn,
      dec_button:    slider_btn,
      border:        0f32,
      border_color:  RGBAColor::new(0, 0, 0),
      cursor_size:   Vec2F32::same(16f32),
      padding:       Vec2F32::same(2f32),
      spacing:       Vec2F32::same(2f32),
      show_buttons:  false,
      bar_height:    8f32,
      rounding:      0f32,
    };

    let progress = StyleProgress {
      normal:              StyleItem::Color(
        table[StyleColors::ColorSlider as usize],
      ),
      hover:               StyleItem::Color(
        table[StyleColors::ColorSlider as usize],
      ),
      active:              StyleItem::Color(
        table[StyleColors::ColorSlider as usize],
      ),
      cursor_normal:       StyleItem::Color(
        table[StyleColors::ColorSliderCursor as usize],
      ),
      cursor_hover:        StyleItem::Color(
        table[StyleColors::ColorSliderCursorHover as usize],
      ),
      cursor_active:       StyleItem::Color(
        table[StyleColors::ColorSliderCursorActive as usize],
      ),
      border_color:        RGBAColor::new(0, 0, 0),
      cursor_border_color: RGBAColor::new(0, 0, 0),
      padding:             Vec2F32::same(4f32),
      rounding:            0f32,
      border:              0f32,
      cursor_rounding:     0f32,
      cursor_border:       0f32,
    };

    let scroll_btn = StyleButton {
      normal:          StyleItem::Color(RGBAColor::new(40, 40, 40)),
      hover:           StyleItem::Color(RGBAColor::new(42, 42, 42)),
      active:          StyleItem::Color(RGBAColor::new(44, 44, 44)),
      border_color:    RGBAColor::new(65, 65, 65),
      text_background: RGBAColor::new(40, 40, 40),
      text_normal:     RGBAColor::new(175, 175, 175),
      text_hover:      RGBAColor::new(175, 175, 175),
      text_active:     RGBAColor::new(175, 175, 175),
      padding:         Vec2F32::same(4f32),
      image_padding:   Vec2F32::same(0f32),
      touch_padding:   Vec2F32::same(0f32),
      text_alignment:  TextAlignment::Centered,
      border:          1f32,
      rounding:        0f32,
    };

    let scroll = StyleScrollbar {
      normal:              StyleItem::Color(
        table[StyleColors::ColorScrollbar as usize],
      ),
      hover:               StyleItem::Color(
        table[StyleColors::ColorScrollbar as usize],
      ),
      active:              StyleItem::Color(
        table[StyleColors::ColorScrollbar as usize],
      ),
      cursor_normal:       StyleItem::Color(
        table[StyleColors::ColorScrollbarCursor as usize],
      ),
      cursor_hover:        StyleItem::Color(
        table[StyleColors::ColorScrollbarCursorHover as usize],
      ),
      cursor_active:       StyleItem::Color(
        table[StyleColors::ColorScrollbarCursorActive as usize],
      ),
      dec_symbol:          SymbolType::CircleSolid,
      inc_symbol:          SymbolType::CircleSolid,
      border_color:        table[StyleColors::ColorScrollbar as usize],
      cursor_border_color: table[StyleColors::ColorScrollbar as usize],
      padding:             Vec2F32::same(0f32),
      show_buttons:        false,
      border:              0f32,
      rounding:            0f32,
      border_cursor:       0f32,
      rounding_cursor:     0f32,
      inc_button:          scroll_btn,
      dec_button:          scroll_btn,
    };

    let edit = StyleEdit {
      normal:               StyleItem::Color(
        table[StyleColors::ColorEdit as usize],
      ),
      hover:                StyleItem::Color(
        table[StyleColors::ColorEdit as usize],
      ),
      active:               StyleItem::Color(
        table[StyleColors::ColorEdit as usize],
      ),
      cursor_normal:        table[StyleColors::ColorText as usize],
      cursor_hover:         table[StyleColors::ColorText as usize],
      cursor_text_normal:   table[StyleColors::ColorEdit as usize],
      cursor_text_hover:    table[StyleColors::ColorEdit as usize],
      border_color:         table[StyleColors::ColorBorder as usize],
      text_normal:          table[StyleColors::ColorText as usize],
      text_hover:           table[StyleColors::ColorText as usize],
      text_active:          table[StyleColors::ColorText as usize],
      selected_normal:      table[StyleColors::ColorText as usize],
      selected_hover:       table[StyleColors::ColorText as usize],
      selected_text_normal: table[StyleColors::ColorEdit as usize],
      selected_text_hover:  table[StyleColors::ColorEdit as usize],
      scrollbar_size:       Vec2F32::same(10f32),
      scrollbar:            scroll,
      padding:              Vec2F32::same(4f32),
      row_padding:          2f32,
      cursor_size:          4f32,
      border:               1f32,
      rounding:             0f32,
    };

    let property_button = StyleButton {
      normal:          StyleItem::Color(
        table[StyleColors::ColorProperty as usize],
      ),
      hover:           StyleItem::Color(
        table[StyleColors::ColorProperty as usize],
      ),
      active:          StyleItem::Color(
        table[StyleColors::ColorProperty as usize],
      ),
      border_color:    RGBAColor::new(0, 0, 0),
      text_background: table[StyleColors::ColorProperty as usize],
      text_normal:     table[StyleColors::ColorText as usize],
      text_hover:      table[StyleColors::ColorText as usize],
      text_active:     table[StyleColors::ColorText as usize],
      padding:         Vec2F32::same(0f32),
      image_padding:   Vec2F32::same(0f32),
      touch_padding:   Vec2F32::same(0f32),
      text_alignment:  TextAlignment::Centered,
      border:          0f32,
      rounding:        0f32,
    };

    let property_edit = StyleEdit {
      normal:               StyleItem::Color(
        table[StyleColors::ColorProperty as usize],
      ),
      hover:                StyleItem::Color(
        table[StyleColors::ColorProperty as usize],
      ),
      active:               StyleItem::Color(
        table[StyleColors::ColorProperty as usize],
      ),
      border_color:         RGBAColor::new(0, 0, 0),
      cursor_normal:        table[StyleColors::ColorText as usize],
      cursor_hover:         table[StyleColors::ColorText as usize],
      cursor_text_normal:   table[StyleColors::ColorEdit as usize],
      cursor_text_hover:    table[StyleColors::ColorEdit as usize],
      text_normal:          table[StyleColors::ColorText as usize],
      text_hover:           table[StyleColors::ColorText as usize],
      text_active:          table[StyleColors::ColorText as usize],
      selected_normal:      table[StyleColors::ColorText as usize],
      selected_hover:       table[StyleColors::ColorText as usize],
      selected_text_normal: table[StyleColors::ColorEdit as usize],
      selected_text_hover:  table[StyleColors::ColorEdit as usize],
      scrollbar_size:       Vec2F32::same(0f32),
      scrollbar:            scroll,
      padding:              Vec2F32::same(0f32),
      row_padding:          0f32,
      cursor_size:          8f32,
      border:               0f32,
      rounding:             0f32,
    };

    let property = StyleProperty {
      normal:       StyleItem::Color(
        table[StyleColors::ColorProperty as usize],
      ),
      hover:        StyleItem::Color(
        table[StyleColors::ColorProperty as usize],
      ),
      active:       StyleItem::Color(
        table[StyleColors::ColorProperty as usize],
      ),
      border_color: table[StyleColors::ColorBorder as usize],
      label_normal: table[StyleColors::ColorText as usize],
      label_hover:  table[StyleColors::ColorText as usize],
      label_active: table[StyleColors::ColorText as usize],
      sym_left:     SymbolType::TriangleLeft,
      sym_right:    SymbolType::TriangleRight,
      padding:      Vec2F32::same(4f32),
      border:       1f32,
      rounding:     10f32,
      dec_button:   property_button,
      inc_button:   property_button,
      edit:         property_edit,
    };

    let chart = StyleChart {
      background:     StyleItem::Color(table[StyleColors::ColorChart as usize]),
      border_color:   table[StyleColors::ColorBorder as usize],
      selected_color: table[StyleColors::ColorChartColorHighlight as usize],
      color:          table[StyleColors::ColorChartColor as usize],
      padding:        Vec2F32::same(4f32),
      border:         0f32,
      rounding:       0f32,
    };

    let combo_button = StyleButton {
      normal:          StyleItem::Color(
        table[StyleColors::ColorCombo as usize],
      ),
      hover:           StyleItem::Color(
        table[StyleColors::ColorCombo as usize],
      ),
      active:          StyleItem::Color(
        table[StyleColors::ColorCombo as usize],
      ),
      border_color:    RGBAColor::new(0, 0, 0),
      text_background: table[StyleColors::ColorCombo as usize],
      text_normal:     table[StyleColors::ColorText as usize],
      text_hover:      table[StyleColors::ColorText as usize],
      text_active:     table[StyleColors::ColorText as usize],
      padding:         Vec2F32::same(2f32),
      touch_padding:   Vec2F32::same(0f32),
      text_alignment:  TextAlignment::Centered,
      border:          0f32,
      rounding:        0f32,
      image_padding:   Vec2F32::same(0f32),
    };

    let combo = StyleCombo {
      normal:          StyleItem::Color(
        table[StyleColors::ColorCombo as usize],
      ),
      hover:           StyleItem::Color(
        table[StyleColors::ColorCombo as usize],
      ),
      active:          StyleItem::Color(
        table[StyleColors::ColorCombo as usize],
      ),
      border_color:    table[StyleColors::ColorBorder as usize],
      label_normal:    table[StyleColors::ColorText as usize],
      label_hover:     table[StyleColors::ColorText as usize],
      label_active:    table[StyleColors::ColorText as usize],
      sym_normal:      SymbolType::TriangleDown,
      sym_hover:       SymbolType::TriangleDown,
      sym_active:      SymbolType::TriangleDown,
      content_padding: Vec2F32::same(4f32),
      button_padding:  Vec2F32::new(0f32, 4f32),
      spacing:         Vec2F32::new(4f32, 0f32),
      border:          1f32,
      rounding:        0f32,
      button:          combo_button,
      symbol_active:   RGBAColor::new(0, 0, 0),
      symbol_hover:    RGBAColor::new(0, 0, 0),
      symbol_normal:   RGBAColor::new(0, 0, 0),
    };

    let tab_btn = StyleButton {
      normal:          StyleItem::Color(
        table[StyleColors::ColorTabHeader as usize],
      ),
      hover:           StyleItem::Color(
        table[StyleColors::ColorTabHeader as usize],
      ),
      active:          StyleItem::Color(
        table[StyleColors::ColorTabHeader as usize],
      ),
      border_color:    RGBAColor::new(0, 0, 0),
      text_background: table[StyleColors::ColorTabHeader as usize],
      text_normal:     table[StyleColors::ColorText as usize],
      text_hover:      table[StyleColors::ColorText as usize],
      text_active:     table[StyleColors::ColorText as usize],
      padding:         Vec2F32::same(2f32),
      touch_padding:   Vec2F32::same(0f32),
      text_alignment:  TextAlignment::Centered,
      border:          0f32,
      rounding:        0f32,
      image_padding:   Vec2F32::same(0f32),
    };

    let tab_node_btn = StyleButton {
      normal:          StyleItem::Color(
        table[StyleColors::ColorWindow as usize],
      ),
      hover:           StyleItem::Color(
        table[StyleColors::ColorWindow as usize],
      ),
      active:          StyleItem::Color(
        table[StyleColors::ColorWindow as usize],
      ),
      border_color:    RGBAColor::new(0, 0, 0),
      text_background: table[StyleColors::ColorTabHeader as usize],
      text_normal:     table[StyleColors::ColorText as usize],
      text_hover:      table[StyleColors::ColorText as usize],
      text_active:     table[StyleColors::ColorText as usize],
      padding:         Vec2F32::same(2f32),
      touch_padding:   Vec2F32::same(0f32),
      text_alignment:  TextAlignment::Centered,
      border:          0f32,
      rounding:        0f32,
      image_padding:   Vec2F32::same(0f32),
    };

    let tab = StyleTab {
      background:           StyleItem::Color(
        table[StyleColors::ColorTabHeader as usize],
      ),
      border_color:         table[StyleColors::ColorBorder as usize],
      text:                 table[StyleColors::ColorText as usize],
      sym_minimize:         SymbolType::TriangleRight,
      sym_maximize:         SymbolType::TriangleDown,
      padding:              Vec2F32::same(4f32),
      spacing:              Vec2F32::same(4f32),
      indent:               10f32,
      border:               1f32,
      rounding:             0f32,
      tab_maximize_button:  tab_btn,
      tab_minimize_button:  tab_btn,
      node_minimize_button: tab_node_btn,
      node_maximize_button: tab_node_btn,
    };

    let win_btn_close = StyleButton {
      normal:          StyleItem::Color(
        table[StyleColors::ColorHeader as usize],
      ),
      hover:           StyleItem::Color(
        table[StyleColors::ColorHeader as usize],
      ),
      active:          StyleItem::Color(
        table[StyleColors::ColorHeader as usize],
      ),
      border_color:    RGBAColor::new(0, 0, 0),
      text_background: table[StyleColors::ColorHeader as usize],
      text_normal:     table[StyleColors::ColorText as usize],
      text_hover:      table[StyleColors::ColorText as usize],
      text_active:     table[StyleColors::ColorText as usize],
      padding:         Vec2F32::same(0f32),
      touch_padding:   Vec2F32::same(0f32),
      text_alignment:  TextAlignment::Centered,
      border:          0f32,
      rounding:        0f32,
      image_padding:   Vec2F32::same(0f32),
    };

    let win_btn_min = StyleButton {
      normal:          StyleItem::Color(
        table[StyleColors::ColorHeader as usize],
      ),
      hover:           StyleItem::Color(
        table[StyleColors::ColorHeader as usize],
      ),
      active:          StyleItem::Color(
        table[StyleColors::ColorHeader as usize],
      ),
      border_color:    RGBAColor::new(0, 0, 0),
      text_background: table[StyleColors::ColorHeader as usize],
      text_normal:     table[StyleColors::ColorText as usize],
      text_hover:      table[StyleColors::ColorText as usize],
      text_active:     table[StyleColors::ColorText as usize],
      padding:         Vec2F32::same(0f32),
      touch_padding:   Vec2F32::same(0f32),
      text_alignment:  TextAlignment::Centered,
      border:          0f32,
      rounding:        0f32,
      image_padding:   Vec2F32::same(0f32),
    };

    let win_header = StyleWindowHeader {
      align:           StyleHeaderAlign::Right,
      close_symbol:    SymbolType::X,
      minimize_symbol: SymbolType::Minus,
      maximize_symbol: SymbolType::Plus,
      normal:          StyleItem::Color(
        table[StyleColors::ColorHeader as usize],
      ),
      hover:           StyleItem::Color(
        table[StyleColors::ColorHeader as usize],
      ),
      active:          StyleItem::Color(
        table[StyleColors::ColorHeader as usize],
      ),
      label_normal:    table[StyleColors::ColorText as usize],
      label_hover:     table[StyleColors::ColorText as usize],
      label_active:    table[StyleColors::ColorText as usize],
      label_padding:   Vec2F32::same(4f32),
      padding:         Vec2F32::same(4f32),
      spacing:         Vec2F32::same(0f32),
      close_button:    win_btn_close,
      minimize_button: win_btn_min,
    };

    let win = StyleWindow {
      header:                  win_header,
      background:              table[StyleColors::ColorWindow as usize],
      fixed_background:        StyleItem::Color(
        table[StyleColors::ColorWindow as usize],
      ),
      border_color:            table[StyleColors::ColorBorder as usize],
      popup_border_color:      table[StyleColors::ColorBorder as usize],
      combo_border_color:      table[StyleColors::ColorBorder as usize],
      contextual_border_color: table[StyleColors::ColorBorder as usize],
      menu_border_color:       table[StyleColors::ColorBorder as usize],
      group_border_color:      table[StyleColors::ColorBorder as usize],
      tooltip_border_color:    table[StyleColors::ColorBorder as usize],
      scaler:                  StyleItem::Color(
        table[StyleColors::ColorText as usize],
      ),
      rounding:                0f32,
      spacing:                 Vec2F32::same(4f32),
      scrollbar_size:          Vec2F32::same(10f32),
      min_size:                Vec2F32::same(64f32),
      combo_border:            1f32,
      contextual_border:       1f32,
      menu_border:             1f32,
      group_border:            1f32,
      tooltip_border:          1f32,
      popup_border:            1f32,
      border:                  2f32,
      min_row_height_padding:  8f32,
      padding:                 Vec2F32::same(4f32),
      group_padding:           Vec2F32::same(4f32),
      popup_padding:           Vec2F32::same(4f32),
      combo_padding:           Vec2F32::same(4f32),
      contextual_padding:      Vec2F32::same(4f32),
      menu_padding:            Vec2F32::same(4f32),
      tooltip_padding:         Vec2F32::same(4f32),
    };
  }
}

// impl std::default::Default for Style {
//   fn default() -> Style {}
// }

struct StackSize {}

impl StackSize {
  pub const BUTTON_BEHAVIOR_STACK_SIZE: usize = 8;
  pub const COLOR_STACK_SIZE: usize = 32;
  pub const FLAGS_STACK_SIZE: usize = 32;
  pub const FLOAT_STACK_SIZE: usize = 32;
  pub const FONT_STACK_SIZE: usize = 8;
  pub const STYLE_ITEM_STACK_SIZE: usize = 16;
  pub const VECTOR_STACK_SIZE: usize = 16;
}

#[derive(Copy, Clone, Debug)]
pub struct ConfigStackElement<T>
where
  T: Copy + Clone + std::fmt::Debug,
{
  pub address:   *mut T,
  pub old_value: T,
}

macro_rules! define_config_stack {
  ($name:ident, $tp:ty, $size:expr) => {
    #[derive(Copy, Clone, Debug)]
    pub struct $name {
      pub head:     i32,
      pub elements: [ConfigStackElement<$tp>; $size],
    }

    impl $name {}
  };
}

define_config_stack!(
  ConfigStackStyleItem,
  StyleItem,
  StackSize::STYLE_ITEM_STACK_SIZE
);
define_config_stack!(ConfigStackFloat, f32, StackSize::FLOAT_STACK_SIZE);
define_config_stack!(ConfigStackVec2, Vec2F32, StackSize::VECTOR_STACK_SIZE);
define_config_stack!(ConfigStackFlags, u32, StackSize::FLAGS_STACK_SIZE);
define_config_stack!(ConfigStackColor, RGBAColor, StackSize::COLOR_STACK_SIZE);
define_config_stack!(ConfigStackFont, Font, StackSize::FONT_STACK_SIZE);
define_config_stack!(
  ConfigStackButtonBehaviour,
  crate::hmi::base::ButtonBehaviour,
  StackSize::BUTTON_BEHAVIOR_STACK_SIZE
);

#[derive(Copy, Clone, Debug)]
pub struct ConfigurationStacks {
  pub style_items:       ConfigStackStyleItem,
  pub floats:            ConfigStackFloat,
  pub vectors:           ConfigStackVec2,
  pub flags:             ConfigStackFlags,
  pub colors:            ConfigStackColor,
  pub fonts:             ConfigStackFont,
  pub button_behaviours: ConfigStackButtonBehaviour,
}
