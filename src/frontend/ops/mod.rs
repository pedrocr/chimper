extern crate conrod_core;
pub use conrod_core::{widget, color, Colorable, Borderable, UiCell, Positionable, Sizeable, Widget, Labelable};
pub use conrod_core::widget::id::Id as WidgetId;
use conrod_core::text::Justify;

extern crate imagepipe;
pub use self::imagepipe::PipelineOps;

use crate::frontend::main::*;

mod rawinput;
mod tolab;
mod basecurve;
mod transform;

pub fn draw_gui(chimper: &mut Chimper, ui: &mut UiCell) {
  let mut ops = chimper.ops.lock().unwrap();
  let ids = match chimper.ids {
    Some(ref mut ids) => ids,
    None => {unreachable!()},
  };

  if let Some((_, ref mut ops)) = *ops {
    let mut voffset = 0.0;
    let mut numop = 0;

    macro_rules! draw_op {
      ($name:expr, $module:ident, $selected:expr) => {
        if ids.ops_headers.len() < numop + 1 {
          ids.ops_headers.resize(numop+1, &mut ui.widget_id_generator());
          ids.ops_settings.resize(numop+1, &mut ui.widget_id_generator());
        }

        for _ in widget::Button::new()
          .label($name)
          .w_of(ids.setcont)
          .h(30.0)
          .top_left_with_margins_on(ids.setcont, voffset, 0.0)
          .set(ids.ops_headers[numop], ui)
        {
          if chimper.selected_op == $selected {
            chimper.selected_op = SelectedOp::None;
          } else {
            chimper.selected_op = $selected;
          }
        }
        voffset += 30.0;
        if chimper.selected_op == $selected {
          widget::Canvas::new()
            .w_of(ids.setcont)
            .h(0.0)
            .color(color::GREY)
            .border(0.0)
            .top_left_with_margins_on(ids.setcont, voffset, 0.0)
            .set(ids.ops_settings[numop], ui);
          let contid = ids.ops_settings[numop].clone();
          voffset += $module::draw_gui(ids, ui, ops, contid);
        }
        numop += 1;
      };
    }

    draw_op!("raw input",  rawinput,  SelectedOp::RawInput);
    draw_op!("colorspace", tolab,     SelectedOp::ToLab);
    draw_op!("basecurve",  basecurve, SelectedOp::Basecurve);
    draw_op!("transform",  transform, SelectedOp::Transform);

    assert!(voffset < 1000.0); // shut up the compiler about the last assignment never being read
    assert!(numop < 1000); // shut up the compiler about the last assignment never being read
  }
}
