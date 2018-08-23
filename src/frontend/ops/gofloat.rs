use frontend::ops::*;

pub fn draw_gui(ids: &mut ChimperIds, ui: &mut UiCell, ops: &mut PipelineOps, id: WidgetId) -> (bool, f64) {
  let mut needs_update = false;

  ids.op_gofloat.resize(2, &mut ui.widget_id_generator());
  let id_imgtype = ids.op_gofloat[0];
  let id_label_imgtype = ids.op_gofloat[1];

  widget::primitive::text::Text::new("Image Type")
    .w_h(150.0, 30.0)
    .top_left_with_margins_on(id, 6.0, 12.0)
    .set(id_label_imgtype, ui)
  ;
  let imgtype = ops.gofloat.is_cfa as usize;
  for event in widget::drop_down_list::DropDownList::new(&["Grayscale", "CFA"], Some(imgtype))
    .w_h(130.0, 30.0)
    .top_left_with_margins_on(id, 6.0, 156.0)
    .set(id_imgtype, ui)
  {
    ops.gofloat.is_cfa = event != 0;
    if !ops.gofloat.is_cfa {
      // If it's already grayscale set a flat WB
      ops.level.wb_coeffs = [1.0, 1.0, 1.0, 1.0];
      // And since it's grayscale it's already sRGB D65
      ops.tolab.cam_to_xyz = [
        [0.4124564, 0.3575761, 0.1804375, 0.0],
        [0.2126729, 0.7151522, 0.0721750, 0.0],
        [0.0193339, 0.1191920, 0.9503041, 0.0],
      ];
    }
    needs_update = true;
  }

  (needs_update, 100.0)
}
