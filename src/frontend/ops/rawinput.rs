use frontend::ops::*;

static PATTERNS: [&str; 11] = [
  "Monochrome",
  "RGGB",
  "GRBG",
  "GBRG",
  "BGGR",
  "Xtrans1",
  "Xtrans2",
  "Xtrans3",
  "ERBG",
  "RGEB",
  "EBGR4",
];

fn get_pattern(patnum: usize) -> String {
  match PATTERNS[patnum] {
    "Xtrans1" => "GBGGRGRGRBGBGBGGRGGRGGBGBGBRGRGRGGBG",
    "Xtrans2" => "GGRGGBGGBGGRBRGRBGGGBGGRGGRGGBRBGBRG",
    "Xtrans3" => "RBGBRGGGRGGBGGBGGRBRGRBGGGBGGRGGRGGB",
    "EBGR4"   => "EBGRBERGEBRGBEGR",
    pat       => pat,
  }.to_string()
}

fn get_patnum(ops: &PipelineOps) -> Option<usize> {
  if !ops.gofloat.is_cfa {
    return Some(0)
  }
  for i in 0..(PATTERNS.len()) {
    if get_pattern(i) == ops.demosaic.cfa {
      return Some(i)
    }
  }
  None
}

pub fn draw_gui(ids: &mut ChimperIds, ui: &mut UiCell, ops: &mut PipelineOps, id: WidgetId) -> (bool, f64) {
  let mut needs_update = false;

  ids.op_rawinput.resize(50, &mut ui.widget_id_generator());

  let mut voffset = 36.0 * 0.5;
  macro_rules! label {
    ($width:expr, $xpos:expr, $widget:expr, $name: expr, $justify:expr) => {
      widget::primitive::text::Text::new($name)
        .justify($justify)
        .w_h($width, 30.0)
        .top_left_with_margins_on(id, voffset+3.0, $xpos)
        .set($widget, ui)
      ;
    };
  }
  macro_rules! left_label {
    ($widget:expr, $name: expr) => {
      label!(140.0, 0.0, $widget, $name, Justify::Right);
    };
  }
  macro_rules! divider_label {
    ($widget:expr, $name: expr) => {
      label!(100.0, 0.0, $widget, $name, Justify::Right);
    };
  }

  divider_label!(ids.op_rawinput[0], "Demosaic");
  voffset += 36.0 * 1.5;
  left_label!(ids.op_rawinput[1], "Filter");
  for event in widget::drop_down_list::DropDownList::new(&PATTERNS, get_patnum(ops))
    .w_h(140.0, 30.0)
    .top_left_with_margins_on(id, voffset, 150.0)
    .set(ids.op_rawinput[2], ui)
  {
    if event == 0 {
      ops.gofloat.is_cfa = false;
    } else {
      ops.gofloat.is_cfa = true;
      ops.demosaic.cfa = get_pattern(event);
    }
    needs_update = true;
  }

  macro_rules! textbox_num_input {
    ($xpos:expr, $ypos:expr, $widget:expr, $value:expr, $typ:ty) => {
      for event in widget::text_box::TextBox::new(&($value.to_string()))
        .center_justify()
        .w_h(80.0, 30.0)
        .top_left_with_margins_on(id, $ypos, $xpos)
        .set($widget, ui)
      {
        if let widget::text_box::Event::Update(val) = event {
          if val.trim() == "" {
            $value = 0 as $typ;
          }
          if let Ok(val) = val.parse::<$typ>() {
            $value = val;
          }
        }
        needs_update = true;
      }
    };
  }

  voffset += 36.0 * 1.5;
  divider_label!(ids.op_rawinput[3], "Crops");
  voffset += 36.0 * 0.5;
  voffset += 36.0;
  left_label!(ids.op_rawinput[4], "Top");
  textbox_num_input!(150.0, voffset, ids.op_rawinput[5], ops.gofloat.crop_top, usize);

  voffset += 36.0;
  left_label!(ids.op_rawinput[6], "Left/Right");
  textbox_num_input!(150.0, voffset, ids.op_rawinput[7], ops.gofloat.crop_left, usize);
  textbox_num_input!(250.0, voffset, ids.op_rawinput[8], ops.gofloat.crop_right, usize);

  voffset += 36.0;
  left_label!(ids.op_rawinput[9], "Bottom");
  textbox_num_input!(150.0, voffset, ids.op_rawinput[10], ops.gofloat.crop_bottom, usize);

  macro_rules! slider_input {
    ($xpos:expr, $ypos:expr, $widget:expr, $value:expr, $min:expr, $max:expr, $typ:ty) => {
      for event in widget::slider::Slider::new($value, $min, $max)
        .w_h(170.0, 30.0)
        .top_left_with_margins_on(id, $ypos, $xpos)
        .set($widget, ui)
      {
        $value = event;
        needs_update = true;
      }
    };
  }

  voffset += 36.0 * 1.5;
  divider_label!(ids.op_rawinput[11], "Levels");
  voffset += 36.0 * 1.5;
  label!(80.0, 150.0, ids.op_rawinput[12], "Min", Justify::Center);
  label!(20.0, 250.0, ids.op_rawinput[13], "0", Justify::Left);
  label!(170.0, 250.0, ids.op_rawinput[14], "Multiplier", Justify::Center);
  label!(20.0, 400.0, ids.op_rawinput[15], "4", Justify::Right);
  label!(80.0, 440.0, ids.op_rawinput[16], "Max", Justify::Center);
  let mut idnum = 17;
  macro_rules! range_widget {
    ($name:expr, $idx:expr) => {
      idnum += 4;
      left_label!(ids.op_rawinput[idnum], $name);
      textbox_num_input!(150.0, voffset, ids.op_rawinput[idnum+1], ops.level.blacklevels[$idx], f32);
      slider_input!(250.0, voffset, ids.op_rawinput[idnum+2], ops.level.wb_coeffs[$idx], 0.0, 4.0, f32);
      textbox_num_input!(440.0, voffset, ids.op_rawinput[idnum+3], ops.level.whitelevels[$idx], f32);

      voffset += 36.0;
    };
  }

  voffset += 36.0;
  range_widget!("Red",     0);
  range_widget!("Green",   1);
  range_widget!("Blue",    2);
  range_widget!("Emerald", 3);

  voffset += 36.0 * 0.5;

  (needs_update, voffset)
}
