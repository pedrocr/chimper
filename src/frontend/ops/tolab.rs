use crate::frontend::ops::*;
use imagepipe::color_conversions::*;

static MIN_TEMP: f32 = 2000.0;
static MAX_TEMP: f32 = 20000.0;

static MIN_TINT: f32 = 8000.0;
static MAX_TINT: f32 = 20000.0;

pub fn temp_tint_image() -> ((u32, u32), Vec<u8>) {
  let width = 500;
  let height = 500;
  let mut data = vec![0 as u8; width*height*4];
  for (row, line) in data.chunks_exact_mut(width*4).enumerate() {
    let rowpos = 1.0 - row as f32 / height as f32;
    let tint = MIN_TINT + rowpos * (MAX_TINT - MIN_TINT);
    for (col, pixout) in line.chunks_exact_mut(4).enumerate() {
      let colpos = col as f32 / width as f32;
      let temp = MIN_TEMP + colpos * (MAX_TEMP - MIN_TEMP);
      let (r, g, b) = temp_tint_to_rgb(temp, tint);
      pixout[0] = output8bit(apply_srgb_gamma(r));
      pixout[1] = output8bit(apply_srgb_gamma(g));
      pixout[2] = output8bit(apply_srgb_gamma(b));
      pixout[3] = 255;
    }
  }
  ((width as u32, height as u32), data)
}

pub fn draw_gui(chimper: &mut Chimper, ui: &mut UiCell, id: WidgetId) -> f64 {
  let ids = &mut chimper.ids;
  let ops = if let Some(ref mut ops) = chimper.ops { ops } else {unreachable!()};
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

  let (otemp, otint) = ops.tolab.get_temp();
  label!(500.0, 80.0, &format!("Temperature {}K", otemp as u32), Justify::Center);
  voffset += 36.0;

  voffset += 150.0;
  label!(60.0, 10.0, &format!("Tint\n{}", otint as u32), Justify::Center);
  voffset -= 150.0;

  widget::Image::new(chimper.temp_tint_image_id)
    .w_h(500.0, 300.0)
    .top_left_with_margins_on(id, voffset, 80.0)
    .set(new_widget!(), ui);

  for (temp, tint) in widget::XYPad::new(otemp, MIN_TEMP, MAX_TEMP, otint, MIN_TINT, MAX_TINT)
    .w_h(500.0, 300.0)
    .top_left_with_margins_on(id, voffset, 80.0)
    .value_font_size(0)
    .color(conrod_core::color::Color::Rgba(1.0,1.0,1.0,0.0))
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
