use crate::hmi::{
  base::ButtonBehaviour,
  commands::CommandBuffer,
  input::Input,
  panel::Panel,
  style::{ConfigurationStacks, Style},
  vertex_output::DrawList,
  window::Window,
};

struct Consts {}

impl Consts {
  const VALUE_PAGE_CAPACITY: usize = 48;
}

#[derive(Copy, Clone)]
pub struct Table {
  pub seq:    u32,
  pub size:   u32,
  pub keys:   [u32; Consts::VALUE_PAGE_CAPACITY],
  pub values: [u32; Consts::VALUE_PAGE_CAPACITY],
}

enum PageData {
  Tbl(Table),
  Pan(Panel),
  Win(Window),
}

pub struct UiContext<'a> {
  pub input:             Input,
  pub style:             Style,
  pub last_widget_state: u32,
  pub button_behviour:   ButtonBehaviour,
  pub stacks:            ConfigurationStacks,
  pub delta_time_sec:    f32,
  draw_list:             DrawList<'a>,
  // TODO: text edit support
  overlay: CommandBuffer,
  // windows
  build:       i32,
  window_list: Vec<Window>,
  active:      Option<usize>,
  current:     Option<usize>,
  count:       u32,
  seq:         u32,
}

impl<'a> UiContext<'a> {
  // pub fn new(font: Font) -> Self {

  // }
}
