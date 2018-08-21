extern crate conrod;
pub use conrod::{widget, color, Colorable, Borderable, UiCell, Positionable, Sizeable, Widget, Labelable};
pub use conrod::widget::id::Id as WidgetId;
extern crate imagepipe;
pub use self::imagepipe::PipelineOps;

use frontend::main::*;

mod gofloat;
mod demosaic;
mod level;
mod tolab;
mod basecurve;
mod transform;

pub fn draw_gui(chimper: &mut Chimper, ui: &mut UiCell) -> bool {
  let mut needs_update = false;
  let mut ops = chimper.ops.lock().unwrap();
    let ids = match chimper.ids {
    Some(ref ids) => ids,
    None => {unreachable!()},
  };

  if let Some((_, ref mut ops)) = *ops {
    let mut voffset = 0.0;

    macro_rules! draw_op {
      ($name:expr, $module:ident, $selected:expr, $idheader:expr, $idbutton:expr, $idcontent:expr) => {
        for _ in widget::Button::new()
          .label($name)
          .w_of(ids.setcont)
          .h(30.0)
          .top_left_with_margins_on(ids.setcont, voffset, 0.0)
          .set($idbutton, ui)
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
            .set($idcontent, ui);
          let (nupdate, vsize) = $module::draw_gui(ids, ui, ops);
          needs_update =  nupdate || needs_update;
          voffset += vsize;
        }
      };
    }

    draw_op!("input", gofloat, SelectedOp::GoFloat, ids.op_gofloat_header, ids.op_gofloat_title, ids.op_gofloat);
    draw_op!("demosaic", demosaic, SelectedOp::Demosaic, ids.op_demosaic_header, ids.op_demosaic_title, ids.op_demosaic);
    draw_op!("levels", level, SelectedOp::Level, ids.op_level_header, ids.op_level_title, ids.op_level);
    draw_op!("colorspace", tolab, SelectedOp::ToLab, ids.op_tolab_header, ids.op_tolab_title, ids.op_tolab);
    draw_op!("basecurve", basecurve, SelectedOp::Basecurve, ids.op_basecurve_header, ids.op_basecurve_title, ids.op_basecurve);
    draw_op!("transform", transform, SelectedOp::Transform, ids.op_transform_header, ids.op_transform_title, ids.op_transform);

    assert!(voffset < 1000.0); // shut up the compiler about the last assignment never being read
  }

  needs_update
}
