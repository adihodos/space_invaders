use crate::math::{rectangle::RectangleF32, vec2::Vec2F32};
use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Copy, Clone, Debug, FromPrimitive, ToPrimitive)]
pub enum KeyId {
  KeyNone,
  KeyShift,
  KeyCtrl,
  KeyDel,
  KeyEnter,
  KeyTab,
  KeyBackspace,
  KeyCopy,
  KeyCut,
  KeyPaste,
  KeyUp,
  KeyDown,
  KeyLeft,
  KeyRight,
  // Shortcuts: text field
  KeyTextInsertMode,
  KeyTextReplaceMode,
  KeyTextResetMode,
  KeyTextLineStart,
  KeyTextLineEnd,
  KeyTextStart,
  KeyTextEnd,
  KeyTextUndo,
  KeyTextRedo,
  KeyTextSelectAll,
  KeyTextWordLeft,
  KeyTextWordRight,
  // Shortcuts: scrollbar
  KeyScrollStart,
  KeyScrollEnd,
  KeyScrollDown,
  KeyScrollUp,
  KeyMax,
}

#[derive(Copy, Clone, Debug, FromPrimitive, ToPrimitive)]
pub enum MouseButtonId {
  ButtonLeft,
  ButtonMiddle,
  ButtonRight,
  ButtonDouble,
  ButtonMax,
}

#[derive(Copy, Debug, Clone)]
pub struct MouseButton {
  pub down:        bool,
  pub clicked:     u32,
  pub clicked_pos: Vec2F32,
}

impl MouseButton {
  pub fn new() -> MouseButton {
    MouseButton {
      down:        false,
      clicked:     0,
      clicked_pos: Vec2F32::same(0f32),
    }
  }
}

#[derive(Copy, Debug, Clone)]
pub struct MouseState {
  pub buttons:      [MouseButton; MouseButtonId::ButtonMax as usize],
  pub pos:          Vec2F32,
  pub prev:         Vec2F32,
  pub delta:        Vec2F32,
  pub scroll_delta: Vec2F32,
  pub grab:         bool,
  pub grabbed:      bool,
  pub ungrab:       bool,
}

impl MouseState {
  pub fn new() -> Self {
    Self {
      buttons:      [MouseButton::new(); MouseButtonId::ButtonMax as usize],
      pos:          Vec2F32::same(0f32),
      prev:         Vec2F32::same(0f32),
      delta:        Vec2F32::same(0f32),
      scroll_delta: Vec2F32::same(0f32),
      grab:         false,
      grabbed:      false,
      ungrab:       false,
    }
  }
}

#[derive(Copy, Debug, Clone)]
pub struct KeyState {
  pub down:    bool,
  pub clicked: u32,
}

impl KeyState {
  pub fn new() -> Self {
    Self {
      down:    false,
      clicked: 0,
    }
  }
}

#[derive(Copy, Debug, Clone)]
pub struct KeyboardState {
  pub keys:     [KeyState; KeyId::KeyMax as usize],
  pub text:     [char; KeyboardState::INPUT_MAX as usize],
  pub text_len: i32,
}

impl KeyboardState {
  pub const INPUT_MAX: u32 = 16;

  pub fn new() -> Self {
    Self {
      keys:     [KeyState::new(); KeyId::KeyMax as usize],
      text:     [0u8 as char; KeyboardState::INPUT_MAX as usize],
      text_len: 0,
    }
  }
}

#[derive(Copy, Debug, Clone)]
pub struct Input {
  pub keyboard: KeyboardState,
  pub mouse:    MouseState,
}

impl Input {
  pub fn new() -> Input {
    Input {
      keyboard: KeyboardState::new(),
      mouse:    MouseState::new(),
    }
  }

  pub fn begin(&mut self) {
    self
      .mouse
      .buttons
      .iter_mut()
      .for_each(|btn_state| btn_state.clicked = 0);

    self.keyboard.text_len = 0;
    self.mouse.scroll_delta = Vec2F32::same(0f32);
    self.mouse.prev = self.mouse.pos;
    self.mouse.delta = Vec2F32::same(0f32);

    self
      .keyboard
      .keys
      .iter_mut()
      .for_each(|key_state| key_state.clicked = 0);
  }

  pub fn end(&mut self) {
    if self.mouse.grab {
      self.mouse.grab = false;
    }

    if self.mouse.ungrab {
      self.mouse.grabbed = false;
      self.mouse.ungrab = false;
      self.mouse.grab = false;
    }
  }

  pub fn motion(&mut self, x: i32, y: i32) {
    self.mouse.pos.x = x as f32;
    self.mouse.pos.y = y as f32;
    self.mouse.delta = self.mouse.pos - self.mouse.prev;
  }

  pub fn key(&mut self, key: KeyId, down: bool) {
    self.keyboard.keys[key as usize].clicked += 1;
    self.keyboard.keys[key as usize].down = down;
  }

  pub fn button(&mut self, id: MouseButtonId, x: i32, y: i32, down: bool) {
    let btn = &mut self.mouse.buttons[id as usize];
    if btn.down == down {
      return;
    }

    btn.clicked_pos = Vec2F32::new(x as f32, y as f32);
    btn.down = down;
    btn.clicked += 1;
  }

  pub fn scroll(&mut self, val: Vec2F32) {
    self.mouse.scroll_delta += val;
  }

  pub fn glyph(&mut self, glyph: char) {
    if self.keyboard.text_len < KeyboardState::INPUT_MAX as i32 {
      self.keyboard.text[self.keyboard.text_len as usize] = glyph;
      self.keyboard.text_len += 1;
    }
  }

  pub fn glyph_ascii(&mut self, glyph: u8) {
    self.glyph(glyph as char);
  }

  pub fn has_mouse_click(&self, id: MouseButtonId) -> bool {
    let btn = &self.mouse.buttons[id as usize];
    btn.clicked != 0 && btn.down == false
  }

  pub fn has_mouse_down(&self, id: MouseButtonId) -> bool {
    let btn = &self.mouse.buttons[id as usize];
    btn.down 
  }

  pub fn has_mouse_button_pressed(&self, id: MouseButtonId) -> bool {
    let btn = &self.mouse.buttons[id as usize];
    btn.clicked != 0
  }

  pub fn has_mouse_click_in_rect(
    &self,
    id: MouseButtonId,
    b: &RectangleF32,
  ) -> bool {
    let btn = &self.mouse.buttons[id as usize];
    b.contains_point(btn.clicked_pos.x, btn.clicked_pos.y)
  }

  pub fn has_mouse_click_down_in_rect(
    &self,
    id: MouseButtonId,
    b: &RectangleF32,
    down: bool,
  ) -> bool {
    let btn = &self.mouse.buttons[id as usize];
    self.has_mouse_click_in_rect(id, b) && btn.down == down
  }

  pub fn is_mouse_click_in_rect(
    &self,
    id: MouseButtonId,
    b: &RectangleF32,
  ) -> bool {
    let btn = &self.mouse.buttons[id as usize];
    self.has_mouse_click_down_in_rect(id, b, false) && btn.clicked != 0
  }

  pub fn is_mouse_click_down_in_rect(
    &self,
    id: MouseButtonId,
    b: &RectangleF32,
    down: bool,
  ) -> bool {
    let btn = &self.mouse.buttons[id as usize];
    self.has_mouse_click_down_in_rect(id, b, down) && btn.clicked != 0
  }

  pub fn any_mouse_click_in_rect(&self, b: &RectangleF32) -> bool {
    num::ToPrimitive::to_i32(&MouseButtonId::ButtonMax).map_or(
      false,
      |last_btn| {
        (0 .. last_btn).any(|btn_id| {
          num::FromPrimitive::from_i32(btn_id)
            .map_or(false, |btn: MouseButtonId| {
              self.is_mouse_click_in_rect(btn, b)
            })
        })
      },
    )
  }

  pub fn is_mouse_hovering_rect(&self, r: &RectangleF32) -> bool {
    r.contains_point(self.mouse.pos.x, self.mouse.pos.y)
  }

  pub fn is_mouse_prev_hovering_rect(&self, r: &RectangleF32) -> bool {
    r.contains_point(self.mouse.prev.x, self.mouse.prev.y)
  }

  pub fn mouse_clicked(&self, id: MouseButtonId, r: &RectangleF32) -> bool {
    self.is_mouse_hovering_rect(r) && self.is_mouse_click_in_rect(id, r)
  }

  pub fn is_mouse_down(&self, id: MouseButtonId) -> bool {
    self.mouse.buttons[id as usize].down
  }

  pub fn is_mouse_pressed(&self, id: MouseButtonId) -> bool {
    let btn = &self.mouse.buttons[id as usize];
    btn.down && btn.clicked != 0
  }

  pub fn is_mouse_released(&self, id: MouseButtonId) -> bool {
    let btn = &self.mouse.buttons[id as usize];
    btn.down && btn.clicked != 0
  }

  pub fn is_key_pressed(&self, key: KeyId) -> bool {
    let k = &self.keyboard.keys[key as usize];
    (k.down && k.clicked != 0) || (!k.down && k.clicked >= 2)
  }

  pub fn is_key_released(&self, key: KeyId) -> bool {
    let k = &self.keyboard.keys[key as usize];
    (!k.down && k.clicked != 0) || (k.down && k.clicked >= 2)
  }

  pub fn is_key_down(&self, key: KeyId) -> bool {
    let k = &self.keyboard.keys[key as usize];
    k.down
  }
}
