extern crate conrod;
use conrod::{widget, Positionable, Sizeable, Widget};
extern crate imagepipe;

use frontend::main::*;

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

pub fn draw_gui(chimper: &mut Chimper, ui: &mut conrod::UiCell) -> bool {
  let mut needs_update = false;
  let mut ops = chimper.ops.lock().unwrap();
    let ids = match chimper.ids {
    Some(ref ids) => ids,
    None => {unreachable!()},
  };

  if let Some((_, ref mut ops)) = *ops {
    let orientation = ops.transform.orientation.to_u16() as usize;
    for event in widget::drop_down_list::DropDownList::new(&ORIENTATION_NAMES, Some(orientation))
      .w_h(130.0, 30.0)
      .top_left_with_margin_on(ids.setcont, 6.0)
      .set(ids.dropdown, ui)
    {
      ops.transform.orientation = imagepipe::Orientation::from_u16(event as u16);
      needs_update = true;
    }
  }

  needs_update
}
