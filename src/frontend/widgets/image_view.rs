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

static MIN_SIZE: f64 = 0.01;

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

    let mut event = None;
    if let Some(crops) = crops {
      let mut crop_top = crops.0;
      let mut crop_right = crops.1;
      let mut crop_bottom = crops.2;
      let mut crop_left = crops.3;

      let [width, height] = rect.dim();
      let handle_size = if width > height { width } else { height } * 0.1;
      let iwidth = width - handle_size * 2.0;
      let iheight = height - handle_size * 2.0;
      let range_x = (0.0 - (handle_size / iwidth), 1.0 + (handle_size / iwidth));
      let range_y = (0.0 - (handle_size / iheight), 1.0 + (handle_size / iheight));

      let mut highlight = ResizeMode::None;
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
          macro_rules! drag_calc {
            ($mode:ident, $crop:ident, $crop_opposite:ident, $drag:expr) => {{
              if drag.mode.$mode() {
                $crop += $drag;
                if $crop < 0.0 {$crop = 0.0};
                let steal = -(1.0 - $crop - $crop_opposite - MIN_SIZE);
                if steal > 0.0 {
                  $crop_opposite -= steal;
                }
                if $crop_opposite < 0.0 {
                  $crop += $crop_opposite;
                  $crop_opposite = 0.0;
                }
                if $crop < 0.0 {$crop = 0.0};
              }
            }}
          }
          drag_calc!(left, crop_left, crop_right, delta_x);
          drag_calc!(right, crop_right, crop_left, -delta_x);
          drag_calc!(top, crop_top, crop_bottom, delta_y);
          drag_calc!(bottom, crop_bottom, crop_top, -delta_y);
          if !mouse.buttons.left().is_down() {
            // We're no longer clicking so reset the state
            state.update(|state| state.drag = None);
            event = Some((crop_top, crop_right, crop_bottom, crop_left));
          }
        } else {
          if new_x < 0.0+crop_left && new_y < 0.0+crop_top {
            highlight = ResizeMode::TopLeft;
          } else if new_x < 0.0+crop_left && new_y > 1.0-crop_bottom {
            highlight = ResizeMode::BottomLeft;
          } else if new_x > 1.0-crop_right && new_y < 0.0+crop_top {
            highlight = ResizeMode::TopRight;
          } else if new_x > 1.0-crop_right && new_y > 1.0-crop_bottom {
            highlight = ResizeMode::BottomRight;
          } else if new_x < 0.0+crop_left {
            highlight = ResizeMode::Left;
          } else if new_y < 0.0+crop_top {
            highlight = ResizeMode::Top;
          } else if new_x > 1.0-crop_right {
            highlight = ResizeMode::Right;
          } else if new_y > 1.0-crop_bottom {
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
        .top_left_with_margins_on(id, 0.0, iwidth * crop_left)
        .graphics_for(id)
        .color(if highlight.left() {color_highlight} else {color})
        .set(state.ids.handles[0], ui);
      widget::primitive::shape::rectangle::Rectangle::fill([handle_size, height])
        .top_right_with_margins_on(id, 0.0, iwidth * crop_right)
        .graphics_for(id)
        .color(if highlight.right() {color_highlight} else {color})
        .set(state.ids.handles[1], ui);
      widget::primitive::shape::rectangle::Rectangle::fill([width, handle_size])
        .top_left_with_margins_on(id, iheight * crop_top, 0.0)
        .graphics_for(id)
        .color(if highlight.top() {color_highlight} else {color})
        .set(state.ids.handles[2], ui);
      widget::primitive::shape::rectangle::Rectangle::fill([width, handle_size])
        .bottom_left_with_margins_on(id, iheight * crop_bottom, 0.0)
        .graphics_for(id)
        .color(if highlight.bottom() {color_highlight} else {color})
        .set(state.ids.handles[3], ui);
    } else {
      // We weren't initialized in crop mode so just show the image
      let [width, height] = rect.dim();
      widget::Image::new(image_id)
        .middle_of(id)
        .w_h(width, height)
        .graphics_for(id)
        .set(state.ids.image, ui);
    }
    event
  }
}
