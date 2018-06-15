extern crate chimper;

#[macro_use] extern crate conrod;
use conrod::{widget, Colorable, Positionable, Sizeable, Borderable, Widget, color};
use conrod::backend::glium::glium;
use std::env;
use std::sync::{Arc, Mutex};
extern crate crossbeam;
extern crate image;

widget_ids!(
struct ChimperIds {
  background, imgcanvas, dragcanvas, setcanvas, settop, setcont, raw_image, chimper, filenav
});

struct Chimper<'a> {
  dragwidth: f64,
  sidewidth: f64,
  imagepadding: f64,
  use_sidepane: bool,
  logoid: conrod::image::Id,
  ids: Option<ChimperIds>,
  imap: &'a Mutex<ImageMapping>,
  file: Option<String>,
  directory: std::path::PathBuf,
}

impl<'a> Chimper<'a> {
  fn new(logoid: conrod::image::Id, imap: &'a Mutex<ImageMapping>,) -> Self {
    Self {
      dragwidth: 10.0,
      sidewidth: 600.0,
      use_sidepane: true,
      imagepadding: 20.0,
      logoid,
      ids: None,
      imap,
      file: None,
      directory: env::current_dir().unwrap(),
    }
  }
}

struct ImageMapping {
  id: Option<(String, usize)>,
  img: Option<(conrod::image::Id,u32,u32)>,
  oldrawid: Option<conrod::image::Id>,
  alldone: bool,
}

impl<'a> chimper::window::ChimperApp for Chimper<'a> {
  fn initialize(&mut self, ui: &mut conrod::Ui) {
    self.ids = Some(ChimperIds::new(ui.widget_id_generator()));
  }

  fn draw_gui(&mut self, ui: &mut conrod::Ui, evproxy: &glium::glutin::EventsLoopProxy) -> bool {
    let mut needs_update = false;

    let ids = match self.ids {
      Some(ref ids) => ids,
      None => unreachable!(),
    };

    let sidewidth = self.sidewidth * ((self.use_sidepane as u8) as f64);
    {
      let ui = &mut ui.set_widgets();

      // Construct our main `Canvas` tree.
      widget::Canvas::new().flow_right(&[
        (ids.imgcanvas, widget::Canvas::new().color(color::CHARCOAL).border(0.0)),
        (ids.dragcanvas, widget::Canvas::new().length(self.dragwidth).color(color::BLACK).border(0.0)),
        (ids.setcanvas, widget::Canvas::new().length(sidewidth).border(0.0).flow_down(&[
          (ids.settop, widget::Canvas::new().color(color::GREY).length(100.0).border(0.0)),
          (ids.setcont, widget::Canvas::new().color(color::GREY).border(0.0)),
        ])),
      ]).border(0.0).set(ids.background, ui);

      let size = chimper::cache::smallest_size(ui.win_w as usize, ui.win_h as usize);
      let img = {
        match self.file {
          None => None,
          Some(ref file) => {
            let newid = Some(((*file).clone(), size));
            let mut imap = self.imap.lock().unwrap();
            if imap.id != newid {
              imap.id = newid;
              imap.img = None;
              imap.alldone = false;
              evproxy.wakeup().is_ok();
              None
            } else {
              imap.img
            }
          },
        }
      };

      if let Some((rawid,maxw,maxh)) = img {
        let scale = (maxw as f64)/(maxh as f64);
        let mut width = (ui.w_of(ids.imgcanvas).unwrap() - self.imagepadding).min(maxw as f64);
        let mut height = (ui.h_of(ids.imgcanvas).unwrap() - self.imagepadding).min(maxh as f64);
        if width/height > scale {
          width = height * scale;
        } else {
          height = width / scale;
        }
        widget::Image::new(rawid)
          .w_h(width, height)
          .middle_of(ids.imgcanvas)
          .set(ids.raw_image, ui);
      }

      if sidewidth > 0.0 {
        widget::Image::new(self.logoid)
          .w_h(78.0, 88.0)
          .top_right_with_margin_on(ids.settop, 6.0)
          .set(ids.chimper, ui);
      }

      let directory = self.directory.as_path();
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
                self.file = Some(path.to_str().unwrap().to_string());
                needs_update = true;
              }
            }
          },
          _ => {},
        }
      }
    }

    needs_update
  }

  fn process_event(&mut self, event: &conrod::event::Input) {
    match *event {
      conrod::event::Input::Press(conrod::input::Button::Keyboard(conrod::input::Key::Tab)) => {
        self.use_sidepane = !self.use_sidepane;
      },
      _ => (),
    }
  }
}


fn main() {
  let mut window = chimper::window::ChimperWindow::new("Chimper", 1200, 800);
  let logoid = window.load_texture(load_image(chimper::logo::random()));

  let icache = chimper::cache::ImageCache::new();
  let imap = Mutex::new(ImageMapping {
    id: None,
    img: None,
    oldrawid: None,
    alldone: true,
  });

  crossbeam::scope(|scope| {
    let mut chimp = Chimper::new(logoid, &imap);
    window.run(&mut chimp, |display, _rederer, image_map, evproxy| {
      //Load images if needed
      let image = {
        let imap = imap.lock().unwrap();
        if imap.alldone {
          Arc::new(None)
        } else {
          if let Some(ref id) = imap.id {
            icache.get(id.0.clone(), id.1, scope, evproxy)
          } else {
            Arc::new(None)
          }
        }
      };

      if let Some(ref imgbuf) = *image {
        let dims = (imgbuf.width as u32, imgbuf.height as u32);
        let raw_image = glium::texture::RawImage2d::from_raw_rgb_reversed(&imgbuf.data, dims);
        let img = glium::texture::SrgbTexture2d::with_format(
          display,
          raw_image,
          glium::texture::SrgbFormat::U8U8U8,
          glium::texture::MipmapsOption::NoMipmap
        ).unwrap();
        let rawid = image_map.insert(img);
        let mut imap = imap.lock().unwrap();
        if let Some(id) = imap.oldrawid {
          image_map.remove(id);
        }
        imap.img = Some((rawid, dims.0, dims.1));
        imap.oldrawid = Some(rawid);
        imap.alldone = true;
        true // cause a redraw
      } else {
        false
      }
    });
  });
}

// Load the image from a file
pub fn load_image(buf: &[u8]) -> glium::texture::RawImage2d<u8> {
  let img = image::load_from_memory(buf).unwrap().to_rgba();
  let dims = img.dimensions();
  glium::texture::RawImage2d::from_raw_rgba_reversed(&img.into_raw(), dims)
}
