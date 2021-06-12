use crate::frontend::ops::*;

pub fn draw_gui(ids: &mut ChimperIds, ui: &mut UiCell, ops: &mut PipelineOps, id: WidgetId) -> f64 {
  let mut numids = 0;
  macro_rules! new_widget {
    () => {{
      numids += 1;
      if ids.op_tolab.len() < numids {
        ids.op_tolab.resize(numids, &mut ui.widget_id_generator());
      }
      ids.op_tolab[numids-1]
    }}
  }

  let mut voffset = 36.0 * 0.25;
  macro_rules! label {
    ($width:expr, $xpos:expr, $name: expr, $justify:expr) => {
      widget::primitive::text::Text::new($name)
        .justify($justify)
        .w_h($width, 30.0)
        .top_left_with_margins_on(id, voffset+3.0, $xpos)
        .set(new_widget!(), ui)
      ;
    };
  }

  label!(500.0, 60.0, "Temperature", Justify::Center);
  voffset += 36.0;

  voffset += 150.0;
  label!(40.0, 10.0, "Tint", Justify::Center);
  voffset -= 150.0;

  let (otemp, otint) = ops.tolab.get_temp();
  for (temp, tint) in widget::XYPad::new(otemp, 2000.0, 20000.0, otint, 8000.0, 20000.0)
    .w_h(500.0, 300.0)
    .top_left_with_margins_on(id, voffset, 60.0)
    .value_font_size(16)
    .set(new_widget!(), ui)
  {
    let delta = 10.0;
    if (temp - otemp).abs() > delta || (tint - otint).abs() > delta {
      log::debug!("Setting temp/tint to {}/{} from {}/{}", temp, tint, otemp, otint);
      ops.tolab.set_temp(temp, tint);
    }
  }
  voffset += 300.0 + 36.0 * 0.5;

  let mut altered = false;
  macro_rules! slider_input {
    ($name:expr, $value:expr, $min:expr, $max:expr) => {
      label!(140.0, 0.0, $name, Justify::Right);
      for event in widget::slider::Slider::new($value as f32, $min as f32, $max as f32)
        .w_h(300.0, 30.0)
        .top_left_with_margins_on(id, voffset, 150.0)
        .set(new_widget!(), ui)
      {
        $value = event;
        altered = true;
      }
      label!(100.0, 460.0, &($value.to_string()), Justify::Left);
      voffset += 36.0;
    };
  }

  let eps = 0.01;
  let has_emerald =
    ops.tolab.cam_to_xyz[0][3].abs() > eps ||
    ops.tolab.cam_to_xyz[1][3].abs() > eps ||
    ops.tolab.cam_to_xyz[2][3].abs() > eps;
  let coeffs = ops.tolab.wb_coeffs;
  let (mut red, mut blue, mut emerald) = (coeffs[0], coeffs[2], coeffs[3]);

  slider_input!("Red", red, 0.0, 5.0);
  slider_input!("Blue", blue, 0.0, 5.0);
  if has_emerald {
    slider_input!("Emerald", emerald, 0.0, 5.0);
  }
  if altered {
    ops.tolab.wb_coeffs = [red, 1.0, blue, emerald];
  }

  voffset += 36.0 * 0.5;

  voffset
}
