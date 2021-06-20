//! Used for the main image display

use conrod_core::utils::map_range;
use conrod_core::widget;
use conrod_core::{Colorable, Sizeable, Positionable, Widget};
use conrod_core::color::RED;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ResizeMode {
  None,
  Left,
  TopLeft,
  Top,
  TopRight,
  Right,
  BottomRight,
  Bottom,
  BottomLeft,
}

impl ResizeMode {
  fn top(&self) -> bool {
    *self == ResizeMode::Top || *self == ResizeMode::TopLeft || *self == ResizeMode::TopRight
  }
  fn bottom(&self) -> bool {
    *self == ResizeMode::Bottom || *self == ResizeMode::BottomLeft || *self == ResizeMode::BottomRight
  }
  fn left(&self) -> bool {
    *self == ResizeMode::Left || *self == ResizeMode::TopLeft || *self == ResizeMode::BottomLeft
  }
  fn right(&self) -> bool {
    *self == ResizeMode::Right || *self == ResizeMode::TopRight || *self == ResizeMode::BottomRight
  }
}

#[derive(WidgetCommon)]
pub struct ImageView {
  #[conrod(common_builder)]
  common: widget::CommonBuilder,
  style: Style,
  image_id: conrod_core::image::Id,
  crops: Option<(f64,f64,f64,f64)>,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle)]
pub struct Style {
}

struct Ids {
  image: conrod_core::widget::Id,
  handles: conrod_core::widget::id::List,
}

impl Ids {
  pub fn new(mut generator: conrod_core::widget::id::Generator) -> Self {
    let mut handles = conrod_core::widget::id::List::new();
    handles.resize(4, &mut generator);
    Ids {
      image: generator.next(),
      handles,
    }
  }
}

#[derive(Copy, Clone, Debug)]
struct Drag {
  x: f64,
  y: f64,
  mode: ResizeMode,
}

pub struct State {
  ids: Ids,
  drag: Option<Drag>,
}

impl<'a> ImageView {
  /// Build a new XYPad widget.
  pub fn new(image_id: conrod_core::image::Id, crops: Option<(f64, f64, f64, f64)>) -> Self {
    Self {
      common: widget::CommonBuilder::default(),
      style: Style::default(),
      image_id,
      crops,
    }
  }
}

impl Widget for ImageView {
  type State = State;
  type Style = Style;
  type Event = Option<(f64,f64,f64,f64)>;

  fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
    // Save an extra for hover
    let ids = Ids::new(id_gen);
    State {
      ids,
      drag: None,
    }
  }

  fn style(&self) -> Self::Style {
    self.style.clone()
  }

  fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
    let widget::UpdateArgs {
      id,
      state,
      rect,
      ui,
      ..
    } = args;

    let Self {
      image_id,
      crops,
      ..
    } = self;

    let mut crop_top = 0.0;
    let mut crop_right = 0.0;
    let mut crop_bottom = 0.0;
    let mut crop_left = 0.0;
    if let Some(crops) = crops {
      crop_top = crops.0;
      crop_right = crops.1;
      crop_bottom = crops.2;
      crop_left = crops.3;
    }

    let [width, height] = rect.dim();
    let handle_size = if width > height { width } else { height } * 0.1;
    let iwidth = width - handle_size * 2.0;
    let iheight = height - handle_size * 2.0;
    let range_x = (0.0 - (handle_size / iwidth), 1.0 + (handle_size / iwidth));
    let range_y = (0.0 - (handle_size / iheight), 1.0 + (handle_size / iheight));

    let mut highlight = ResizeMode::None;
    let mut event = None;
    if let Some(mouse) = ui.widget_input(id).mouse() {
      let mouse_abs_xy = mouse.abs_xy();
      let clamped_x = rect.x.clamp_value(mouse_abs_xy[0]);
      let clamped_y = rect.y.clamp_value(mouse_abs_xy[1]);
      let (l, r, b, t) = rect.l_r_b_t();
      let new_x = map_range(clamped_x, l, r, range_x.0, range_x.1);
      let new_y = map_range(clamped_y, t, b, range_y.0, range_y.1);
      if let Some(drag) = state.drag {
        highlight = drag.mode;
        let delta_x = new_x - drag.x;
        let delta_y = new_y - drag.y;
        if drag.mode.left() {
          crop_left += delta_x;
          if crop_left < 0.0 {crop_left = 0.0};
        }
        if drag.mode.right() {
          crop_right -= delta_x;
          if crop_right < 0.0 {crop_right = 0.0};
        }
        if drag.mode.top() {
          crop_top += delta_y;
          if crop_top < 0.0 {crop_top = 0.0};
        }
        if drag.mode.bottom() {
          crop_bottom -= delta_y;
          if crop_bottom < 0.0 {crop_bottom = 0.0};
        }
        event = Some((crop_top, crop_right, crop_bottom, crop_left));
        if !mouse.buttons.left().is_down() {
          // We're no longer clicking so reset the state
          state.update(|state| state.drag = None);
        } else {
          state.update(|state| state.drag = Some(Drag {
            x: new_x,
            y: new_y,
            mode: drag.mode,
          }));
        }
      } else {
        if new_x < 0.0 && new_y < 0.0 {
          highlight = ResizeMode::TopLeft;
        } else if new_x < 0.0 && new_y > 1.0 {
          highlight = ResizeMode::BottomLeft;
        } else if new_x > 1.0 && new_y < 0.0 {
          highlight = ResizeMode::TopRight;
        } else if new_x > 1.0 && new_y > 1.0 {
          highlight = ResizeMode::BottomRight;
        } else if new_x < 0.0 {
          highlight = ResizeMode::Left;
        } else if new_y < 0.0 {
          highlight = ResizeMode::Top;
        } else if new_x > 1.0 {
          highlight = ResizeMode::Right;
        } else if new_y > 1.0 {
          highlight = ResizeMode::Bottom;
        }

        if mouse.buttons.left().is_down() && highlight != ResizeMode::None {
          // We're in the first click so initiate a drag
          state.update(|state| state.drag = Some(Drag {
            x: new_x,
            y: new_y,
            mode: highlight,
          }));
        }
      }
    }

    let color = RED.with_alpha(0.1);
    let color_highlight = RED.with_alpha(0.5);

    widget::Image::new(image_id)
      .middle_of(id)
      .w_h(iwidth, iheight)
      .graphics_for(id)
      .set(state.ids.image, ui);
    widget::primitive::shape::rectangle::Rectangle::fill([handle_size, height])
      .top_left_with_margins_on(id, 0.0, width * crop_left)
      .graphics_for(id)
      .color(if highlight.left() {color_highlight} else {color})
      .set(state.ids.handles[0], ui);
    widget::primitive::shape::rectangle::Rectangle::fill([handle_size, height])
      .top_right_with_margins_on(id, 0.0, width * crop_right)
      .graphics_for(id)
      .color(if highlight.right() {color_highlight} else {color})
      .set(state.ids.handles[1], ui);
    widget::primitive::shape::rectangle::Rectangle::fill([width, handle_size])
      .top_left_with_margins_on(id, height * crop_top, 0.0)
      .graphics_for(id)
      .color(if highlight.top() {color_highlight} else {color})
      .set(state.ids.handles[2], ui);
    widget::primitive::shape::rectangle::Rectangle::fill([width, handle_size])
      .bottom_left_with_margins_on(id, height * crop_bottom, 0.0)
      .graphics_for(id)
      .color(if highlight.bottom() {color_highlight} else {color})
      .set(state.ids.handles[3], ui);

    event
  }
}
