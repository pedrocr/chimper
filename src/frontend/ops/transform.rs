use frontend::ops::*;

static ORIENTATION_NAMES: [&str; 8] = [
  "Normal",
  "HorizontalFlip",
  "Rotate180",
  "VerticalFlip",
  "Transpose",
  "Rotate90",
  "Transverse",
  "Rotate270",
];

pub fn draw_gui(ids: &mut ChimperIds, ui: &mut UiCell, ops: &mut PipelineOps, id: WidgetId) -> (bool, f64) {
  let mut needs_update = false;

  ids.op_transform.resize(1, &mut ui.widget_id_generator());
  let id_orientation = ids.op_transform[0];

  let orientation = ops.transform.orientation.to_u16() as usize;
  for event in widget::drop_down_list::DropDownList::new(&ORIENTATION_NAMES, Some(orientation))
    .w_h(130.0, 30.0)
    .top_left_with_margin_on(id, 6.0)
    .set(id_orientation, ui)
  {
    ops.transform.orientation = imagepipe::Orientation::from_u16(event as u16);
    needs_update = true;
  }

  (needs_update, 40.0)
}
