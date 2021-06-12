//! Used for displaying and controlling a 2D point on a cartesian plane within a given range.

use conrod_core::utils::map_range;
use conrod_core::widget;
use conrod_core::{Borderable, Color, Colorable, Positionable, Scalar, Widget};

/// Used for displaying and controlling a 2D point on a cartesian plane within a given range.
///
/// Its reaction is triggered when the value is updated or if the mouse button is released while
/// the cursor is above the rectangle.
#[derive(WidgetCommon)]
pub struct CurveEditor {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    range_x: (f32,f32),
    range_y: (f32,f32),
    points: Vec<(f32,f32)>,
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

struct Ids {
    rectangle: conrod_core::widget::Id,
    points: conrod_core::widget::id::List,
}

impl Ids {
    pub fn new(mut generator: conrod_core::widget::id::Generator, npoints: usize) -> Self {
        let mut points = conrod_core::widget::id::List::new();
        points.resize(npoints, &mut generator);
        Ids {
            rectangle: generator.next(),
            points,
        }
    }
}

/// The state of the XYPad.
pub struct State {
    ids: Ids,
}

impl<'a> CurveEditor {
    /// Build a new XYPad widget.
    pub fn new(range_x: (f32,f32), range_y: (f32, f32), points: &[(f32, f32)]) -> Self {
        Self {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            range_x,
            range_y,
            points: points.into(),
            enabled: true,
        }
    }

    builder_methods! {
        pub line_thickness { style.line_thickness = Some(Scalar) }
        pub enabled { enabled = bool }
    }
}

impl Widget for CurveEditor {
    type State = State;
    type Style = Style;
    type Event = Option<Vec<(f32, f32)>>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        let ids = Ids::new(id_gen, self.points.len());
        State {
            ids,
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
            range_x,
            range_y,
            points,
            ..
        } = self;

        let border = style.border(ui.theme());
        let inner_rect = rect.pad(border);

        let (x, y) = points[0];

        let mut new_x = x;
        let mut new_y = y;
        if let Some(mouse) = ui.widget_input(id).mouse() {
            if mouse.buttons.left().is_down() {
                let mouse_abs_xy = mouse.abs_xy();
                let clamped_x = inner_rect.x.clamp_value(mouse_abs_xy[0]);
                let clamped_y = inner_rect.y.clamp_value(mouse_abs_xy[1]);
                let (l, r, b, t) = inner_rect.l_r_b_t();
                new_x = map_range(clamped_x, l, r, range_x.0, range_x.1);
                new_y = map_range(clamped_y, b, t, range_y.0, range_y.1);
            }
        }

        // If the value across either axis has changed, produce an event.
        let event = if x != new_x || y != new_y {
            Some(vec![(new_x, new_y)])
        } else {
            None
        };

        let color = style.color(ui.theme());
        let line_color = style.line_color(ui.theme()).with_alpha(1.0);

        // The backdrop **BorderedRectangle** widget.
        let dim = rect.dim();

        let border = style.border(ui.theme());
        let border_color = style.border_color(ui.theme());
        widget::BorderedRectangle::new(dim)
            .middle_of(id)
            .graphics_for(id)
            .color(color)
            .border(border)
            .border_color(border_color)
            .set(state.ids.rectangle, ui);

        // The points in the curve
        for (i,(x,y)) in points.into_iter().enumerate() {
          let xpos = x as f64 * dim[0] - 5.0;
          let ypos = dim[1] - y as f64 * dim[1] - 5.0;
          widget::primitive::shape::circle::Circle::fill(5.0)
            .top_left_with_margins_on(state.ids.rectangle, ypos, xpos)
            .graphics_for(id)
            .color(line_color)
            .set(state.ids.points[i], ui);
        }

        event
    }
}

impl Colorable for CurveEditor {
    builder_method!(color { style.color = Some(Color) });
}

impl Borderable for CurveEditor {
    builder_methods! {
        border { style.border = Some(Scalar) }
        border_color { style.border_color = Some(Color) }
    }
}
