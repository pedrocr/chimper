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
    /// The thickness of the line
    #[conrod(default = "2.0")]
    pub line_thickness: Option<Scalar>,
    /// The radius of the points
    #[conrod(default = "5.0")]
    pub point_radius: Option<Scalar>,
}

struct Ids {
    rectangle: conrod_core::widget::Id,
    line: conrod_core::widget::Id,
    points: conrod_core::widget::id::List,
}

impl Ids {
    pub fn new(mut generator: conrod_core::widget::id::Generator, npoints: usize) -> Self {
        let mut points = conrod_core::widget::id::List::new();
        points.resize(npoints, &mut generator);
        Ids {
            rectangle: generator.next(),
            line: generator.next(),
            points,
        }
    }
}

/// The state of the XYPad.
pub struct State {
    ids: Ids,
    currpoint: Option<usize>,
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
        pub point_radius { style.point_radius = Some(Scalar) }
        pub enabled { enabled = bool }
    }
}

impl Widget for CurveEditor {
    type State = State;
    type Style = Style;
    type Event = Option<Vec<(f32, f32)>>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        // Save an extra for hover
        let ids = Ids::new(id_gen, self.points.len() + 1);
        State {
            ids,
            currpoint: None,
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

        // Resize the Ids to however many points we will be displaying
        state.update(|state| {
            let id_gen = ui.widget_id_generator();
            state.ids = Ids::new(id_gen, points.len() + 1);
        });

        let border = style.border(ui.theme());
        let inner_rect = rect.pad(border);

        let mut event = None;
        let mut hover = None;
        if let Some(mouse) = ui.widget_input(id).mouse() {
            let mouse_abs_xy = mouse.abs_xy();
            let clamped_x = inner_rect.x.clamp_value(mouse_abs_xy[0]);
            let clamped_y = inner_rect.y.clamp_value(mouse_abs_xy[1]);
            let (l, r, b, t) = inner_rect.l_r_b_t();
            let new_x = map_range(clamped_x, l, r, range_x.0, range_x.1);
            let new_y = map_range(clamped_y, b, t, range_y.0, range_y.1);
            if mouse.buttons.left().is_down() {
                let mut newpoints = points.clone();
                if let Some(pos) = state.currpoint {
                    // We were already dragging a point, just replace the values
                    // but don't allow going below the previous point or above
                    // the next one
                    let min_x = if pos > 0 { newpoints[pos-1].0 } else { 0.0 };
                    let max_x = if newpoints.len() > 1 && pos < newpoints.len()-1 {
                      newpoints[pos+1].0
                    } else { 1.0 };
                    let new_x = if new_x < min_x { min_x } else { new_x };
                    let new_x = if new_x > max_x { max_x } else { new_x };
                    newpoints[pos] = (new_x, new_y);
                } else {
                    let mut newpos = None;
                    let mut insertpos = 0;
                    // First look for a point that's very close by to grab
                    for &(x,y) in &points {
                        if (x - new_x).abs() < 0.01 && (y - new_y).abs() < 0.01 {
                            newpoints[insertpos] = (new_x, new_y);
                            newpos = Some(insertpos);
                            break;
                        }
                        if x > new_x {
                            // We're past the point we need to add
                            break;
                        }
                        insertpos += 1;
                    }
                    if newpos.is_none() {
                        // We didn't replace a point so we need to add a new one
                        newpoints.insert(insertpos, (new_x, new_y));
                        newpos = Some(insertpos);
                    }
                    state.update(|state| state.currpoint = newpos);
                }
                event = Some(newpoints);
            } else {
                state.update(|state| state.currpoint = None);
                let spline = imagepipe::SplineFunc::new(&points);
                // +-2% feels reasonable for the phantom point display
                // not too small a target and not too far off that it feels strange
                if (spline.interpolate(new_x) - new_y).abs() < 0.02 {
                    hover = Some(new_x);
                }
            }
        }

        let color = style.color(ui.theme());
        let line_color = style.line_color(ui.theme()).with_alpha(1.0);
        let line_thickness = style.line_thickness(ui.theme());
        let point_radius = style.point_radius(ui.theme());

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

        // The line that connects the points
        let spline = imagepipe::SplineFunc::new(&points);
        widget::plot_path::PlotPath::new(0.0, 1.0, 0.0, 1.0, |val| spline.interpolate(val))
            .middle_of(id)
            .graphics_for(id)
            .color(line_color)
            .thickness(line_thickness)
            .set(state.ids.line, ui);

        // The points in the curve
        let mut npoint = 0;
        for (x,y) in points.into_iter() {
          let xpos = x as f64 * dim[0] - point_radius;
          let ypos = dim[1] - y as f64 * dim[1] - point_radius;
          widget::primitive::shape::circle::Circle::fill(point_radius)
            .top_left_with_margins_on(state.ids.rectangle, ypos, xpos)
            .graphics_for(id)
            .color(line_color)
            .set(state.ids.points[npoint], ui);
          npoint += 1;
        }

        // The point on the line the mouse is hovering over, if it exists
        if let Some(x) = hover {
          let y = spline.interpolate(x);
          let xpos = x as f64 * dim[0] - point_radius;
          let ypos = dim[1] - y as f64 * dim[1] - point_radius;
          widget::primitive::shape::circle::Circle::fill(point_radius)
            .top_left_with_margins_on(state.ids.rectangle, ypos, xpos)
            .graphics_for(id)
            .color(line_color.with_alpha(0.8))
            .set(state.ids.points[npoint], ui);
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
