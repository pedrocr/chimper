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
    macro_rules! oplen {
      ($op:expr) => {
        if $op == chimper.selected_op { 100.0 } else { 0.0 };
      };
    }

    macro_rules! draw_op {
      ($name:expr, $module:ident, $selected:expr, $idheader:expr, $idbutton:expr) => {
        for _ in widget::Button::new()
          .label($name)
          .kid_area_wh_of($idheader)
          .middle_of($idheader)
          .set($idbutton, ui)
        {
          if chimper.selected_op == $selected {
            chimper.selected_op = SelectedOp::None;
          } else {
            chimper.selected_op = $selected;
          }
        }
        if chimper.selected_op == $selected {
          needs_update = $module::draw_gui(ids, ui, ops) || needs_update;
        } else {
          
        }
      };
    }

    // Construct our ops canvas sequence
    widget::Canvas::new().flow_down(&[
      (ids.op_gofloat_header, widget::Canvas::new().length(30.0).color(color::GREY).border(1.0)),
      (ids.op_gofloat, widget::Canvas::new().length(oplen!(SelectedOp::GoFloat)).color(color::GREY).border(0.0)),

      (ids.op_demosaic_header, widget::Canvas::new().length(30.0).color(color::GREY).border(1.0)),
      (ids.op_demosaic, widget::Canvas::new().length(oplen!(SelectedOp::Demosaic)).color(color::GREY).border(0.0)),

      (ids.op_level_header, widget::Canvas::new().length(30.0).color(color::GREY).border(1.0)),
      (ids.op_level, widget::Canvas::new().length(oplen!(SelectedOp::Level)).color(color::GREY).border(0.0)),

      (ids.op_tolab_header, widget::Canvas::new().length(30.0).color(color::GREY).border(1.0)),
      (ids.op_tolab, widget::Canvas::new().length(oplen!(SelectedOp::ToLab)).color(color::GREY).border(0.0)),

      (ids.op_basecurve_header, widget::Canvas::new().length(30.0).color(color::GREY).border(1.0)),
      (ids.op_basecurve, widget::Canvas::new().length(oplen!(SelectedOp::Basecurve)).color(color::GREY).border(0.0)),

      (ids.op_transform_header, widget::Canvas::new().length(30.0).color(color::GREY).border(1.0)),
      (ids.op_transform, widget::Canvas::new().length(oplen!(SelectedOp::Transform)).color(color::GREY).border(0.0)),
    ])
      .border(0.0)
      .kid_area_wh_of(ids.setcont)
      .middle_of(ids.setcont)
      .set(ids.oplist, ui)
    ;

    draw_op!("input", gofloat, SelectedOp::GoFloat, ids.op_gofloat_header, ids.op_gofloat_title);
    draw_op!("demosaic", demosaic, SelectedOp::Demosaic, ids.op_demosaic_header, ids.op_demosaic_title);
    draw_op!("levels", level, SelectedOp::Level, ids.op_level_header, ids.op_level_title);
    draw_op!("colorspace", tolab, SelectedOp::ToLab, ids.op_tolab_header, ids.op_tolab_title);
    draw_op!("basecurve", basecurve, SelectedOp::Basecurve, ids.op_basecurve_header, ids.op_basecurve_title);
    draw_op!("transform", transform, SelectedOp::Transform, ids.op_transform_header, ids.op_transform_title);
  }

  needs_update
}
