use crate::frontend::ops::*;

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

pub fn is_unchanged(chimper: &Chimper) -> bool {
  if let Some(ref ops) = chimper.ops {
    let (ops, default_ops) =  ops;
    return ops.gofloat.shash() == default_ops.gofloat.shash() &&
      ops.demosaic.shash() == default_ops.demosaic.shash()
  }
  unreachable!();
}

pub fn reset(chimper: &mut Chimper) {
  if let Some(ref mut ops) = chimper.ops {
    let (ops, default_ops) =  ops;
    ops.gofloat = default_ops.gofloat;
    ops.demosaic = default_ops.demosaic.clone();
    return;
  }
  unreachable!();
}

pub fn draw_gui(chimper: &mut Chimper, ui: &mut UiCell, id: WidgetId) -> f64 {
  let ids = &mut chimper.ids;
  let ops = if let Some((ref mut ops,_)) = chimper.ops { ops } else {unreachable!()};
  let mut numids = 0;
  macro_rules! new_widget {
    () => {{
      numids += 1;
      if ids.op_rawinput.len() < numids {
        ids.op_rawinput.resize(numids, &mut ui.widget_id_generator());
      }
      ids.op_rawinput[numids-1]
    }}
  }

  let mut voffset = 36.0 * 0.5;
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
  macro_rules! left_label {
    ($name: expr) => {
      label!(140.0, 0.0, $name, Justify::Right);
    };
  }
  macro_rules! divider_label {
    ($name: expr) => {
      label!(100.0, 0.0, $name, Justify::Right);
    };
  }

  divider_label!("Demosaic");
  voffset += 36.0 * 1.5;
  left_label!("Filter");
  for event in widget::drop_down_list::DropDownList::new(&PATTERNS, get_patnum(ops))
    .w_h(140.0, 30.0)
    .top_left_with_margins_on(id, voffset, 150.0)
    .set(new_widget!(), ui)
  {
    if event == 0 {
      ops.gofloat.is_cfa = false;
    } else {
      ops.gofloat.is_cfa = true;
      ops.demosaic.cfa = get_pattern(event);
    }
  }

  macro_rules! textbox_num_input {
    ($xpos:expr, $value:expr, $typ:ty) => {
      for event in widget::text_box::TextBox::new(&($value.to_string()))
        .center_justify()
        .w_h(80.0, 30.0)
        .top_left_with_margins_on(id, voffset, $xpos)
        .set(new_widget!(), ui)
      {
        if let widget::text_box::Event::Update(val) = event {
          if val.trim() == "" {
            $value = 0 as $typ;
          }
          if let Ok(val) = val.parse::<$typ>() {
            $value = val;
          }
        }
      }
    };
  }

  voffset += 36.0 * 1.5;
  divider_label!("Crops");
  voffset += 36.0 * 0.5;
  voffset += 36.0;
  left_label!("Top");
  textbox_num_input!(150.0, ops.gofloat.crop_top, usize);

  voffset += 36.0;
  left_label!("Left/Right");
  textbox_num_input!(150.0, ops.gofloat.crop_left, usize);
  textbox_num_input!(250.0, ops.gofloat.crop_right, usize);

  voffset += 36.0;
  left_label!("Bottom");
  textbox_num_input!(150.0, ops.gofloat.crop_bottom, usize);

  voffset += 36.0 * 1.5;
  divider_label!("Levels");
  voffset += 36.0 * 1.5;
  label!(80.0,  150.0, "Min", Justify::Center);
  label!(80.0,  250.0, "Max", Justify::Center);
  macro_rules! range_widget {
    ($name:expr, $idx:expr) => {
      left_label!($name);
      textbox_num_input!(150.0, ops.gofloat.blacklevels[$idx], f32);
      textbox_num_input!(250.0, ops.gofloat.whitelevels[$idx], f32);

      voffset += 36.0;
    };
  }

  voffset += 36.0;
  range_widget!("Red",     0);
  range_widget!("Green",   1);
  range_widget!("Blue",    2);
  range_widget!("Emerald", 3);

  voffset += 36.0 * 0.5;

  voffset
}
