extern crate conrod;
pub use conrod::{widget, UiCell, Positionable, Sizeable, Widget};
extern crate imagepipe;
pub use self::imagepipe::PipelineOps;

use frontend::main::*;

mod transform;

pub fn draw_gui(chimper: &mut Chimper, ui: &mut UiCell) -> bool {
  let mut needs_update = false;
  let mut ops = chimper.ops.lock().unwrap();
    let ids = match chimper.ids {
    Some(ref ids) => ids,
    None => {unreachable!()},
  };

  if let Some((_, ref mut ops)) = *ops {
    needs_update = transform::draw_gui(ids, ui, ops) || needs_update;
  }

  needs_update
}
