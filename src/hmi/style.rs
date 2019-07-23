use crate::{
  hmi::{cursor::Cursor, image::Image, text_engine::Font},
  math::{colors::RGBAColor, vec2::Vec2F32},
};
use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Copy, Clone, Debug, FromPrimitive, ToPrimitive)]
pub enum SymbolType {
  SymbolNone,
  SymbolX,
  SymbolUnderscore,
  SymbolCircleSolid,
  SymbolCircleOutline,
  SymbolRectSolid,
  SymbolRectOutline,
  SymbolTriangleUp,
  SymbolTriangleDown,
  SymbolTriangleLeft,
  SymbolTriangleRight,
  SymbolPlus,
  SymbolMinus,
  SymbolMax,
}

#[derive(Copy, Clone, Debug)]
pub enum StyleItem {
  Img(Image),
  Color(RGBAColor),
}

impl StyleItem {
  fn hide(&mut self) {
    *self = StyleItem::Color(RGBAColor::new_with_alpha(0, 0, 0, 0))
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

  pub fn from_table(table: &[RGBAColor])  {
    // default button
     let text = StyleText {
       color: table[StyleColors::ColorText as usize],
       padding: Vec2F32::same(0f32)
     };

// default text
     let button = StyleButton {
       normal: StyleItem::Color(table[StyleColors::ColorButton as usize] ),
       hover: StyleItem::Color(table[StyleColors::ColorButtonHover as usize]),
       active: StyleItem::Color(table[StyleColors::ColorButtonActive as usize] ),
       border_color: table[StyleColors::ColorBorder as usize],
       text_background: table[StyleColors::ColorButton as usize],
       text_normal: table[StyleColors::ColorText as usize],
       text_hover: table[StyleColors::ColorText as usize],
       text_active: table[StyleColors::ColorText as usize],
       padding: Vec2F32::same(2f32),
       image_padding: Vec2F32::same(0f32),
       touch_padding: Vec2F32::same(0f32),
      //  text_alignment: Text

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
