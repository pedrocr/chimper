extern crate conrod;
use conrod::{widget, Colorable, Positionable, Sizeable, Borderable, Widget, color};
extern crate imagepipe;

use frontend::main::Chimper;

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

pub fn draw_gui(chimper: &mut Chimper, ui: &mut conrod::Ui) -> bool {
  let mut needs_update = false;
  let ui = &mut ui.set_widgets();
  let mut ops = chimper.ops.lock().unwrap();

  let ids = match chimper.ids {
    Some(ref ids) => ids,
    None => {unreachable!()},
  };
  let sidewidth = chimper.sidewidth * ((chimper.use_sidepane as u8) as f64);
  let dragwidth = chimper.dragwidth * ((chimper.use_sidepane as u8) as f64);
  // Construct our main `Canvas` tree.
  widget::Canvas::new().flow_right(&[
    (ids.imgcanvas, widget::Canvas::new().color(color::CHARCOAL).border(0.0)),
    (ids.dragcanvas, widget::Canvas::new().length(dragwidth).color(color::BLACK).border(0.0)),
    (ids.setcanvas, widget::Canvas::new().length(sidewidth).border(0.0).flow_down(&[
      (ids.settop, widget::Canvas::new().color(color::GREY).length(100.0).border(0.0)),
      (ids.setcont, widget::Canvas::new().color(color::GREY).border(0.0)),
    ])),
  ]).border(0.0).set(ids.background, ui);

  if let Some(ref image) = chimper.image {
    let scale = (image.width as f64)/(image.height as f64);
    let mut width = (ui.w_of(ids.imgcanvas).unwrap() - chimper.imagepadding).min(image.width as f64);
    let mut height = (ui.h_of(ids.imgcanvas).unwrap() - chimper.imagepadding).min(image.height as f64);
    if width/height > scale {
      width = height * scale;
    } else {
      height = width / scale;
    }
    widget::Image::new(image.id)
      .w_h(width, height)
      .middle_of(ids.imgcanvas)
      .set(ids.raw_image, ui);
  }

  if sidewidth > 0.0 {
    for _event in widget::Button::image(chimper.logoid)
      .w_h(78.0, 88.0)
      .top_right_with_margin_on(ids.settop, 6.0)
      .set(ids.chimper, ui)
    {
      chimper.sideopt = !chimper.sideopt;
    }

    if chimper.sideopt {
      let directory = chimper.directory.as_path();
      for event in widget::FileNavigator::all(&directory)
        .color(conrod::color::LIGHT_BLUE)
        .font_size(16)
        .kid_area_wh_of(ids.setcont)
        .middle_of(ids.setcont)
        //.show_hidden_files(true)  // Use this to show hidden files
        .set(ids.filenav, ui)
      {
        match event {
          conrod::widget::file_navigator::Event::ChangeSelection(pbuf) => {
            if pbuf.len() > 0 {
              let path = pbuf[0].as_path();
              if path.is_file() {
                eprintln!("Loading file {:?}", path);
                chimper.file = Some(path.to_str().unwrap().to_string());
                needs_update = true;
              }
            }
          },
          _ => {},
        }
      }
    } else {
      let orientation = if let Some((_, ref ops)) = *ops {
        ops.transform.orientation.to_u16() as usize
      } else {
        0
      };

      for event in widget::drop_down_list::DropDownList::new(&ORIENTATION_NAMES, Some(orientation))
        .w_h(130.0, 30.0)
        .top_left_with_margin_on(ids.setcont, 6.0)
        .set(ids.dropdown, ui)
      {
        if let Some((_, ref mut ops)) = *ops {
          ops.transform.orientation = imagepipe::Orientation::from_u16(event as u16);
          needs_update = true;
        };
      }
    }
  }

  needs_update
}
