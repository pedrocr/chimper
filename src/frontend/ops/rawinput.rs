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

  ids.op_rawinput.resize(35, &mut ui.widget_id_generator());
  let id_pattern = ids.op_rawinput[0];
  let id_label_pattern = ids.op_rawinput[1];

  let mut voffset = 6.0;
  widget::primitive::text::Text::new("Filter Pattern")
    .w_h(150.0, 30.0)
    .top_left_with_margins_on(id, voffset, 12.0)
    .set(id_label_pattern, ui)
  ;
  for event in widget::drop_down_list::DropDownList::new(&PATTERNS, get_patnum(ops))
    .w_h(130.0, 30.0)
    .top_left_with_margins_on(id, voffset, 156.0)
    .set(id_pattern, ui)
  {
    if event == 0 {
      ops.gofloat.is_cfa = false;
    } else {
      ops.gofloat.is_cfa = true;
      ops.demosaic.cfa = get_pattern(event);
    }
    needs_update = true;
  }

  macro_rules! label {
    ($parentwidget:expr, $xpos:expr, $ypos:expr, $widget:expr, $name: expr) => {
      widget::primitive::text::Text::new($name)
        .w_h(150.0, 30.0)
        .top_left_with_margins_on($parentwidget, $ypos, $xpos)
        .set($widget, ui)
      ;
    };
  }

  macro_rules! textbox_num_input {
    ($parentwidget:expr, $xpos:expr, $ypos:expr, $widget:expr, $value:expr, $typ:ty) => {
      for event in widget::text_box::TextBox::new(&($value.to_string()))
        .right_justify()
        .w_h(130.0, 30.0)
        .top_left_with_margins_on($parentwidget, $ypos, $xpos)
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

  macro_rules! slider_input {
    ($parentwidget:expr, $xpos:expr, $ypos:expr, $widget:expr, $value:expr, $min:expr, $max:expr, $typ:ty) => {
      for event in widget::slider::Slider::new($value, $min, $max)
        .w_h(300.0, 30.0)
        .top_left_with_margins_on($parentwidget, $ypos, $xpos)
        .set($widget, ui)
      {
        $value = event;
        needs_update = true;
      }
    };
  }

  voffset += 36.0;
  let mut idnum = 0;
  macro_rules! crop_widget {
    ($name:expr, $value:ident) => {
      idnum += 2;
      label!(id, 12.0, voffset, ids.op_rawinput[idnum], $name);
      textbox_num_input!(id, 156.0, voffset, ids.op_rawinput[idnum+1], ops.gofloat.$value, usize);
      voffset += 36.0;
    };
  }

  crop_widget!("Crop Left",   crop_left);
  crop_widget!("Crop Right",  crop_right);
  crop_widget!("Crop Top",    crop_top);
  crop_widget!("Crop Bottom", crop_bottom);

  macro_rules! range_widget {
    ($name:expr, $idx:expr) => {
      idnum += 3;
      label!(id, 12.0, voffset, ids.op_rawinput[idnum], $name);
      textbox_num_input!(id, 156.0, voffset, ids.op_rawinput[idnum+1], ops.level.blacklevels[$idx], f32);
      textbox_num_input!(id, 306.0, voffset, ids.op_rawinput[idnum+2], ops.level.whitelevels[$idx], f32);

      voffset += 36.0;
    };
  }

  range_widget!("Red",     0);
  range_widget!("Green",   1);
  range_widget!("Blue",    2);
  range_widget!("Emerald", 3);

  macro_rules! slide_widget {
    ($name:expr, $idx:expr) => {
      idnum += 3;
      label!(id, 12.0, voffset, ids.op_rawinput[idnum], $name);
      slider_input!(id, 156.0, voffset, ids.op_rawinput[idnum+1], ops.level.wb_coeffs[$idx], 0.0, 3.0, f32);
      voffset += 36.0;
    };
  }

  slide_widget!("Red",     0);
  slide_widget!("Green",   1);
  slide_widget!("Blue",    2);
  slide_widget!("Emerald", 3);

  (needs_update, voffset)
}
