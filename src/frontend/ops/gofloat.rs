use frontend::ops::*;

pub fn draw_gui(ids: &mut ChimperIds, ui: &mut UiCell, ops: &mut PipelineOps, id: WidgetId) -> (bool, f64) {
  let mut needs_update = false;

  ids.op_gofloat.resize(10, &mut ui.widget_id_generator());
  let id_imgtype = ids.op_gofloat[0];
  let id_label_imgtype = ids.op_gofloat[1];

  let mut voffset = 6.0;
  widget::primitive::text::Text::new("Image Type")
    .w_h(150.0, 30.0)
    .top_left_with_margins_on(id, voffset, 12.0)
    .set(id_label_imgtype, ui)
  ;
  let imgtype = ops.gofloat.is_cfa as usize;
  for event in widget::drop_down_list::DropDownList::new(&["Grayscale", "CFA"], Some(imgtype))
    .w_h(130.0, 30.0)
    .top_left_with_margins_on(id, voffset, 156.0)
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

  voffset += 36.0;
  let mut idnum = 0;
  macro_rules! crop_widget {
    ($name:expr, $value:ident) => {
      idnum += 2;
      widget::primitive::text::Text::new($name)
        .w_h(150.0, 30.0)
        .top_left_with_margins_on(id, voffset, 12.0)
        .set(ids.op_gofloat[idnum], ui)
      ;
      for event in widget::text_box::TextBox::new(&ops.gofloat.$value.to_string())
        .right_justify()
        .w_h(130.0, 30.0)
        .top_left_with_margins_on(id, voffset, 156.0)
        .set(ids.op_gofloat[idnum+1], ui)
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
