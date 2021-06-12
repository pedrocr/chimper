use crate::frontend::ops::*;
use imagepipe::color_conversions::*;

static MIN_TEMP: f32 = 2000.0;
static MAX_TEMP: f32 = 20000.0;

static MIN_TINT: f32 = 0.8;
static MAX_TINT: f32 = 1.5;

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

  let mut altered = false;
  let (otemp, otint) = ops.tolab.get_temp();
  let mut temp = otemp;
  let mut tint = otint;
  label!(500.0, 80.0, &format!("Temperature {}K", otemp as u32), Justify::Center);
  voffset += 36.0;

  voffset += 150.0;
  label!(60.0, 10.0, &format!("Tint\n{:.2}", otint), Justify::Center);
  voffset -= 150.0;

  for etemp in widget::slider::Slider::new(otemp, MIN_TEMP, MAX_TEMP)
    .w_h(460.0, 30.0)
    .top_left_with_margins_on(id, voffset, 120.0)
    .set(new_widget!(), ui)
  {
    temp = etemp;
    altered = true;
  }
  voffset += 40.0;

  for etint in widget::slider::Slider::new(otint, MIN_TINT, MAX_TINT)
    .w_h(30.0, 300.0)
    .top_left_with_margins_on(id, voffset, 80.0)
    .set(new_widget!(), ui)
  {
    tint = etint;
    altered = true;
  }

  widget::Image::new(chimper.temp_tint_image_id)
    .w_h(460.0, 300.0)
    .top_left_with_margins_on(id, voffset, 120.0)
    .set(new_widget!(), ui);

  for (etemp, etint) in widget::XYPad::new(otemp, MIN_TEMP, MAX_TEMP, otint, MIN_TINT, MAX_TINT)
    .w_h(460.0, 300.0)
    .top_left_with_margins_on(id, voffset, 120.0)
    .value_font_size(0)
    .color(conrod_core::color::Color::Rgba(1.0,1.0,1.0,0.0))
    .set(new_widget!(), ui)
  {
    temp = etemp;
    tint = etint;
    altered = true;
  }
  voffset += 300.0;

  if altered {
    let deltatemp = 10.0;
    let deltatint = 0.01;
    if (temp - otemp).abs() > deltatemp || (tint - otint).abs() > deltatint {
      log::debug!("Setting temp/tint to {}/{} from {}/{}", temp, tint, otemp, otint);
      ops.tolab.set_temp(temp, tint);
    }
  }

  let eps = 0.002;
  let has_emerald =
    ops.tolab.cam_to_xyz[0][3].abs() > eps ||
    ops.tolab.cam_to_xyz[1][3].abs() > eps ||
    ops.tolab.cam_to_xyz[2][3].abs() > eps;
  let coeffs = ops.tolab.wb_coeffs;
  let (red, green, blue, emerald) = (coeffs[0], coeffs[1], coeffs[2], coeffs[3]);
  let text = if has_emerald {
    format!("Multipliers\nR: {:.2} G: {:.2} B: {:.2} E: {:.2}", red, green, blue, emerald)
  } else {
    format!("Multipliers\nR: {:.2} G: {:.2} B: {:.2}", red, green, blue)
  };
  label!(500.0, 80.0, &text, Justify::Center);

  voffset += 36.0;

  voffset += 36.0 * 0.5;

  voffset
}
