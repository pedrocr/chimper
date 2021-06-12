extern crate conrod_core;
pub use conrod_core::{widget, color, Colorable, Borderable, UiCell, Positionable, Sizeable, Widget, Labelable};
pub use conrod_core::widget::id::Id as WidgetId;
use conrod_core::text::Justify;

extern crate imagepipe;
pub use self::imagepipe::PipelineOps;
pub use self::imagepipe::ImageOp;

use crate::frontend::main::*;

mod rawinput;
pub mod tolab;
mod basecurve;
mod transform;

pub fn draw_gui(chimper: &mut Chimper, ui: &mut UiCell) {
  if chimper.ops.is_some() {
    let mut voffset = 0.0;
    let mut numop = 0;

    macro_rules! draw_op {
      ($name:expr, $module:ident, $selected:expr) => {
        if chimper.ids.ops_headers.len() < numop + 1 {
          chimper.ids.ops_headers.resize(numop+1, &mut ui.widget_id_generator());
          chimper.ids.ops_settings.resize(numop+1, &mut ui.widget_id_generator());
          chimper.ids.ops_resets.resize(numop+1, &mut ui.widget_id_generator());
        }

        for _ in widget::Button::new()
          .label($name)
          .w_of(chimper.ids.setcont)
          .h(30.0)
          .top_left_with_margins_on(chimper.ids.setcont, voffset, 0.0)
          .set(chimper.ids.ops_headers[numop], ui)
        {
          if chimper.selected_op == $selected {
            chimper.selected_op = SelectedOp::None;
          } else {
            chimper.selected_op = $selected;
          }
        }
        if !$module::is_unchanged(chimper) {
          for _ in widget::Button::new()
            .label("Reset")
            .w_h(50.0, 30.0)
            .top_right_with_margins_on(chimper.ids.setcont, voffset, 0.0)
            .set(chimper.ids.ops_resets[numop], ui)
          {
            $module::reset(chimper);
          }
        }
        voffset += 30.0;
        if chimper.selected_op == $selected {
          widget::Canvas::new()
            .w_of(chimper.ids.setcont)
            .h(0.0)
            .color(color::GREY)
            .border(0.0)
            .top_left_with_margins_on(chimper.ids.setcont, voffset, 0.0)
            .set(chimper.ids.ops_settings[numop], ui);
          let contid = chimper.ids.ops_settings[numop].clone();
          voffset += $module::draw_gui(chimper, ui, contid);
        }
        numop += 1;
      };
    }

    draw_op!("raw input",  rawinput,  SelectedOp::RawInput);
    draw_op!("colorspace", tolab,     SelectedOp::ToLab);
    draw_op!("basecurve",  basecurve, SelectedOp::Basecurve);
    draw_op!("transform",  transform, SelectedOp::Transform);

    assert!(voffset < 10000.0); // shut up the compiler about the last assignment never being read
    assert!(numop < 1000); // shut up the compiler about the last assignment never being read
  }
}
