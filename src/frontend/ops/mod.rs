extern crate conrod_core;
pub use conrod_core::{widget, color, Colorable, Borderable, UiCell, Positionable, Sizeable, Widget, Labelable};
pub use conrod_core::widget::id::Id as WidgetId;
use conrod_core::text::Justify;

extern crate imagepipe;
pub use self::imagepipe::PipelineOps;
pub use self::imagepipe::ImageOp;
pub use crate::frontend::widgets::*;

use crate::frontend::main::*;
use crate::backend::export::*;

mod rawinput;
pub mod tolab;
mod basecurve;
mod transform;
mod rotatecrop;

pub fn draw_gui(chimper: &mut Chimper, ui: &mut UiCell) {
  if chimper.ops.is_some() {
    if chimper.selected_op == SelectedOp::RotateCrop {
      // When we enter the crop editing op, initialize that state in the interface
      // and set the crops to 0.0 on the pipeline so we get the full image.
      if chimper.crops.is_none() {
        if let Some(ref mut ops) = chimper.ops {
          chimper.crops = Some((
            ops.0.rotatecrop.crop_top as f64,
            ops.0.rotatecrop.crop_right as f64,
            ops.0.rotatecrop.crop_bottom as f64,
            ops.0.rotatecrop.crop_left as f64,
          ));
          ops.0.rotatecrop.crop_top = 0.0;
          ops.0.rotatecrop.crop_right = 0.0;
          ops.0.rotatecrop.crop_bottom = 0.0;
          ops.0.rotatecrop.crop_left = 0.0;
        }
      }
    } else {
      // When we leave the crop editing op save the state into the pipeline so
      // the changes get applied
      if let Some(ref crops) = chimper.crops {
        if let Some(ref mut ops) = chimper.ops {
          ops.0.rotatecrop.crop_top = crops.0 as f32;
          ops.0.rotatecrop.crop_right = crops.1 as f32;
          ops.0.rotatecrop.crop_bottom = crops.2 as f32;
          ops.0.rotatecrop.crop_left = crops.3 as f32;
        }
        chimper.crops = None;
      }
    }

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
    draw_op!("rotate and crop",  rotatecrop, SelectedOp::RotateCrop);

    for _ in widget::Button::new()
      .label("Export")
      .w_of(chimper.ids.setcont)
      .h(30.0)
      .bottom_left_of(chimper.ids.setcont)
      .set(chimper.ids.ops_export, ui)
    {
      if let Some(ref file) = chimper.file {
        let file = file.clone();
        let ops = if let Some((ref ops, _)) = chimper.ops {
          Some(ops.clone())
        } else {
          None
        };
        chimper.export_request_tx.send(RequestedExport{file, ops}).unwrap();
      } else {
        log::error!("Trying to export with no file selected!");
      }
    }

    assert!(voffset < 10000.0); // shut up the compiler about the last assignment never being read
    assert!(numop < 1000); // shut up the compiler about the last assignment never being read
  }
}
