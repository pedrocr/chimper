use frontend::ops::*;
use self::imagepipe::Rotation;

static ORIENTATION_NAMES: [&str; 4] = [
  "Normal",
  "Rotate 90",
  "Rotate 180",
  "Rotate 270",
];

pub fn draw_gui(ids: &mut ChimperIds, ui: &mut UiCell, ops: &mut PipelineOps, id: WidgetId) -> f64 {
  ids.op_transform.resize(6, &mut ui.widget_id_generator());
  let id_toggle_h = ids.op_transform[0];
  let id_toggle_v = ids.op_transform[1];
  let id_orientation = ids.op_transform[2];
  let id_label_toggle_h = ids.op_transform[3];
  let id_label_toggle_v = ids.op_transform[4];
  let id_label_orientation = ids.op_transform[5];

  widget::primitive::text::Text::new("Flip Horizontally")
    .w_h(150.0, 30.0)
    .top_left_with_margins_on(id, 12.0, 12.0)
    .set(id_label_toggle_h, ui)
  ;
  for event in widget::toggle::Toggle::new(ops.transform.fliph)
    .w_h(50.0, 30.0)
    .label("ON")
    .top_left_with_margins_on(id, 6.0, 156.0)
    .set(id_toggle_h, ui)
  {
    ops.transform.fliph = event;
  }

  widget::primitive::text::Text::new("Flip Vertically")
    .w_h(150.0, 30.0)
    .top_left_with_margins_on(id, 48.0, 12.0)
    .set(id_label_toggle_v, ui)
  ;
  for event in widget::toggle::Toggle::new(ops.transform.flipv)
    .w_h(50.0, 30.0)
    .label("ON")
    .top_left_with_margins_on(id, 42.0, 156.0)
    .set(id_toggle_v, ui)
  {
    ops.transform.flipv = event;
  }

  widget::primitive::text::Text::new("Orientation")
    .w_h(150.0, 30.0)
    .top_left_with_margins_on(id, 84.0, 12.0)
    .set(id_label_orientation, ui)
  ;
  let rotation = match ops.transform.rotation {
    Rotation::Normal    => 0,
    Rotation::Rotate90  => 1,
    Rotation::Rotate180 => 2,
    Rotation::Rotate270 => 3,
  };
  for event in widget::drop_down_list::DropDownList::new(&ORIENTATION_NAMES, Some(rotation))
    .w_h(130.0, 30.0)
    .top_left_with_margins_on(id, 78.0, 156.0)
    .set(id_orientation, ui)
  {
    ops.transform.rotation = match event {
      0 => Rotation::Normal,
      1 => Rotation::Rotate90,
      2 => Rotation::Rotate180,
      3 => Rotation::Rotate270,
      _ => Rotation::Normal,
    };
  }

  114.0
}
