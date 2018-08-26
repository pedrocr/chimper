use frontend::ops::*;

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

  let mut voffset = 36.0 * 0.5;
  let mut altered = false;
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
  macro_rules! slider_input {
    ($name:expr, $value:expr, $min:expr, $max:expr) => {
      label!(140.0, 0.0, $name, Justify::Right);
      for event in widget::slider::Slider::new($value as f32, $min as f32, $max as f32)
        .w_h(300.0, 30.0)
        .top_left_with_margins_on(id, voffset, 150.0)
        .set(new_widget!(), ui)
      {
        $value = event as u32;
        altered = true;
      }
      label!(100.0, 460.0, &($value.to_string()), Justify::Left);
      voffset += 36.0;
    };
  }

  let (mut temp, mut tint) = ops.tolab.get_temp();
  slider_input!("Temperature", temp, 1000, 25000);
  slider_input!("Tint", tint, 1000, 25000);
  if altered {
    ops.tolab.set_temp(temp, tint);
  }
  voffset += 36.0 *0.5;

  voffset
}
