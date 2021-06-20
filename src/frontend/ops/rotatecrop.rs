use crate::frontend::ops::*;

pub fn is_unchanged(chimper: &Chimper) -> bool {
  if let Some(ref ops) = chimper.ops {
    let (ops, default_ops) =  ops;
    if let Some(ref crops) = chimper.crops {
      if *crops != (0.0, 0.0, 0.0, 0.0) {
        return false;
      }
    }
    return ops.rotatecrop.shash() == default_ops.rotatecrop.shash()
  }
  unreachable!();
}

pub fn reset(chimper: &mut Chimper) {
  if let Some(ref mut crops) = chimper.crops {
    *crops = (0.0, 0.0, 0.0, 0.0);
  }
  if let Some(ref mut ops) = chimper.ops {
    let (ops, default_ops) =  ops;
    ops.rotatecrop = default_ops.rotatecrop;
    return;
  }
  unreachable!();
}

pub fn draw_gui(_chimper: &mut Chimper, _ui: &mut UiCell, _id: WidgetId) -> f64 {
  0.0
}
