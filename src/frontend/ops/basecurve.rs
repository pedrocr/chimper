use crate::frontend::ops::*;

pub fn is_unchanged(chimper: &Chimper) -> bool {
  if let Some(ref ops) = chimper.ops {
    let (ops, default_ops) =  ops;
    return ops.basecurve.shash() == default_ops.basecurve.shash()
  }
  unreachable!();
}

pub fn reset(chimper: &mut Chimper) {
  if let Some(ref mut ops) = chimper.ops {
    let (ops, default_ops) =  ops;
    ops.basecurve = default_ops.basecurve.clone();
    return;
  }
  unreachable!();
}

pub fn draw_gui(chimper: &mut Chimper, ui: &mut UiCell, id: WidgetId) -> f64 {
  let ids = &mut chimper.ids;
  let ops = if let Some((ref mut ops,_)) = chimper.ops { ops } else {unreachable!()};
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

  let spline = ops.basecurve.get_spline();
  widget::plot_path::PlotPath::new(0.0, 1.0, 0.0, 1.0, |val| spline.interpolate(val))
    .w_h(500.0, 500.0)
    .top_left_with_margins_on(id, voffset, 50.0)
    .thickness(2.0)
    .color(conrod_core::color::Color::Rgba(0.0,0.0,0.0,1.0))
    .set(new_widget!(), ui);
  let (x, y) = ops.basecurve.points[1].clone();
  for (x, y) in CurveEditor::new((0.0, 1.0), (0.0, 1.0), &[(x,y)])
    .w_h(500.0, 500.0)
    .top_left_with_margins_on(id, voffset, 50.0)
    .color(conrod_core::color::Color::Rgba(1.0,1.0,1.0,0.0))
    .set(new_widget!(), ui)
  {
    ops.basecurve.points[1] = (x, y);
  }  
  
  voffset += 500.0;

  voffset += 36.0 *0.5;

  voffset
}
