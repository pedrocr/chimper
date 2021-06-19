//! Used for displaying and controlling a 2D point on a cartesian plane within a given range.

use conrod_core::utils::map_range;
use conrod_core::widget;
use conrod_core::{Borderable, Color, Colorable, Positionable, Scalar, Widget};

/// Used for displaying and controlling a 2D point on a cartesian plane within a given range.
///
/// Its reaction is triggered when the value is updated or if the mouse button is released while
/// the cursor is above the rectangle.
#[derive(WidgetCommon)]
pub struct SimplerXYPad {
  #[conrod(common_builder)]
  common: widget::CommonBuilder,
  x: f32,
  min_x: f32,
  max_x: f32,
  y: f32,
  min_y: f32,
  max_y: f32,
  style: Style,
  /// Indicates whether the XYPad will respond to user input.
  pub enabled: bool,
}

/// Unique graphical styling for the XYPad.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle)]
pub struct Style {
  /// The color of the XYPad's rectangle.
  #[conrod(default = "theme.shape_color")]
  pub color: Option<Color>,
  /// The color of the XYPad's lines.
  #[conrod(default = "theme.label_color")]
  pub line_color: Option<Color>,
  /// The width of the border surrounding the rectangle.
  #[conrod(default = "theme.border_width")]
  pub border: Option<Scalar>,
  /// The color of the surrounding rectangle border.
  #[conrod(default = "theme.border_color")]
  pub border_color: Option<Color>,
  /// The thickness of the XYPad's crosshair lines.
  #[conrod(default = "2.0")]
  pub line_thickness: Option<Scalar>,
}

widget_ids! {
    struct Ids {
        rectangle,
        h_line,
        v_line,
    }
}

/// The state of the XYPad.
pub struct State {
  ids: Ids,
}

impl<'a> SimplerXYPad {
  /// Build a new XYPad widget.
  pub fn new(x_val: f32, min_x: f32, max_x: f32, y_val: f32, min_y: f32, max_y: f32) -> Self {
    Self {
      common: widget::CommonBuilder::default(),
      style: Style::default(),
      x: x_val,
      min_x: min_x,
      max_x: max_x,
      y: y_val,
      min_y: min_y,
      max_y: max_y,
      enabled: true,
    }
  }

  builder_methods! {
      pub line_thickness { style.line_thickness = Some(Scalar) }
      pub enabled { enabled = bool }
  }
}

impl Widget for SimplerXYPad {
  type State = State;
  type Style = Style;
  type Event = Option<(f32, f32)>;

  fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
    State {
      ids: Ids::new(id_gen),
    }
  }

  fn style(&self) -> Self::Style {
    self.style.clone()
  }

  /// Update the XYPad's cached state.
  fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
    let widget::UpdateArgs {
      id,
      state,
      rect,
      style,
      ui,
      ..
    } = args;
    let Self {
      x,
      min_x,
      max_x,
      y,
      min_y,
      max_y,
      ..
    } = self;

    let border = style.border(ui.theme());
    let inner_rect = rect.pad(border);

    let mut new_x = x;
    let mut new_y = y;
    if let Some(mouse) = ui.widget_input(id).mouse() {
      if mouse.buttons.left().is_down() {
        let mouse_abs_xy = mouse.abs_xy();
        let clamped_x = inner_rect.x.clamp_value(mouse_abs_xy[0]);
        let clamped_y = inner_rect.y.clamp_value(mouse_abs_xy[1]);
        let (l, r, b, t) = inner_rect.l_r_b_t();
        new_x = map_range(clamped_x, l, r, min_x, max_x);
        new_y = map_range(clamped_y, b, t, min_y, max_y);
      }
    }

    // If the value across either axis has changed, produce an event.
    let event = if x != new_x || y != new_y {
      Some((new_x, new_y))
    } else {
      None
    };

    // The backdrop **BorderedRectangle** widget.
    let dim = rect.dim();
    let color = style.color(ui.theme());
    let border = style.border(ui.theme());
    let border_color = style.border_color(ui.theme());
    widget::BorderedRectangle::new(dim)
      .middle_of(id)
      .graphics_for(id)
      .color(color)
      .border(border)
      .border_color(border_color)
      .set(state.ids.rectangle, ui);

    // Crosshair **Line** widgets.
    let (w, h) = inner_rect.w_h();
    let half_w = w / 2.0;
    let half_h = h / 2.0;
    let v_line_x = map_range(new_x, min_x, max_x, -half_w, half_w);
    let h_line_y = map_range(new_y, min_y, max_y, -half_h, half_h);
    let thickness = style.line_thickness(ui.theme());
    let line_color = style.line_color(ui.theme()).with_alpha(1.0);

    let v_line_start = [0.0, -half_h];
    let v_line_end = [0.0, half_h];
    widget::Line::centred(v_line_start, v_line_end)
      .color(line_color)
      .thickness(thickness)
      .x_y_relative_to(id, v_line_x, 0.0)
      .graphics_for(id)
      .parent(id)
      .set(state.ids.v_line, ui);

    let h_line_start = [-half_w, 0.0];
    let h_line_end = [half_w, 0.0];
    widget::Line::centred(h_line_start, h_line_end)
      .color(line_color)
      .thickness(thickness)
      .x_y_relative_to(id, 0.0, h_line_y)
      .graphics_for(id)
      .parent(id)
      .set(state.ids.h_line, ui);

    event
  }
}

impl Colorable for SimplerXYPad {
  builder_method!(color { style.color = Some(Color) });
}

impl Borderable for SimplerXYPad {
  builder_methods! {
      border { style.border = Some(Scalar) }
      border_color { style.border_color = Some(Color) }
  }
}
