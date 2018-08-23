use frontend::ops::*;

static PATTERNS: [&str; 10] = [
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

fn get_patnum(pattern: &str) -> Option<usize> {
  for i in 0..(PATTERNS.len()) {
    if &(get_pattern(i)) == pattern {
      return Some(i)
    }
  }
  None
}

pub fn draw_gui(ids: &mut ChimperIds, ui: &mut UiCell, ops: &mut PipelineOps, id: WidgetId) -> (bool, f64) {
  let mut needs_update = false;

  ids.op_demosaic.resize(2, &mut ui.widget_id_generator());
  let id_pattern = ids.op_demosaic[0];
  let id_label_pattern = ids.op_demosaic[1];

  let mut voffset = 6.0;
  widget::primitive::text::Text::new("Filter Pattern")
    .w_h(150.0, 30.0)
    .top_left_with_margins_on(id, voffset, 12.0)
    .set(id_label_pattern, ui)
  ;
  for event in widget::drop_down_list::DropDownList::new(&PATTERNS, get_patnum(&ops.demosaic.cfa))
    .w_h(130.0, 30.0)
    .top_left_with_margins_on(id, voffset, 156.0)
    .set(id_pattern, ui)
  {
    ops.demosaic.cfa = get_pattern(event);
    needs_update = true;
  }
  voffset += 36.0;

  (needs_update, voffset)
}
