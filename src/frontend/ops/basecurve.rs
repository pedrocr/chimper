use crate::frontend::ops::*;

pub fn draw_gui(chimper: &mut Chimper, ui: &mut UiCell, id: WidgetId) -> f64 {
  let ids = &mut chimper.ids;
  let ops = if let Some(ref mut ops) = chimper.ops { ops } else {unreachable!()};
  let mut numids = 0;
  macro_rules! new_widget {
    () => {{
      numids += 1;
      if ids.op_basecurve.len() < numids {
        ids.op_basecurve.resize(numids, &mut ui.widget_id_generator());
      }
      ids.op_basecurve[numids-1]
    }}
  }

  let mut voffset = 36.0 * 0.5;
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
        $value = event;
      }
      label!(100.0, 460.0, &($value.to_string()), Justify::Left);
      voffset += 36.0;
    };
  }

  slider_input!("Exposure", ops.basecurve.exposure, -5.0, 5.0);
  voffset += 36.0 *0.5;

  voffset
}
