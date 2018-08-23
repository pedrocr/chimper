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

  ids.op_rawinput.resize(10, &mut ui.widget_id_generator());
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

  voffset += 36.0;
  let mut idnum = 0;
  macro_rules! crop_widget {
    ($name:expr, $value:ident) => {
      idnum += 2;
      widget::primitive::text::Text::new($name)
        .w_h(150.0, 30.0)
        .top_left_with_margins_on(id, voffset, 12.0)
        .set(ids.op_rawinput[idnum], ui)
      ;
      for event in widget::text_box::TextBox::new(&ops.gofloat.$value.to_string())
        .right_justify()
        .w_h(130.0, 30.0)
        .top_left_with_margins_on(id, voffset, 156.0)
        .set(ids.op_rawinput[idnum+1], ui)
      {
        if let widget::text_box::Event::Update(val) = event {
          if val.trim() == "" {
            ops.gofloat.$value = 0;
          }
          if let Ok(val) = val.parse::<usize>() {
            ops.gofloat.$value = val;
          }
        }
        needs_update = true;
      }
      voffset += 36.0;
    };
  }

  crop_widget!("Crop Left",   crop_left);
  crop_widget!("Crop Right",  crop_right);
  crop_widget!("Crop Top",    crop_top);
  crop_widget!("Crop Bottom", crop_bottom);


  (needs_update, voffset)
}
