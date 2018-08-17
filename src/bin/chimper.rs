extern crate chimper;

#[macro_use] extern crate conrod;
use conrod::{widget, Colorable, Positionable, Sizeable, Borderable, Widget, color};
use conrod::backend::glium::glium;
use std::env;
use std::sync::Mutex;
extern crate crossbeam_utils;
extern crate image;

widget_ids!(
struct ChimperIds {
  background, imgcanvas, dragcanvas, setcanvas, settop, setcont, raw_image, chimper, filenav, dropdown
});

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

struct Chimper<'a> {
  dragwidth: f64,
  sidewidth: f64,
  imagepadding: f64,
  use_sidepane: bool,
  logoid: conrod::image::Id,
  ids: Option<ChimperIds>,
  imap: &'a Mutex<ImageState>,
  file: Option<String>,
  directory: std::path::PathBuf,
  sideopt: bool,
  orientation: usize,
}

impl<'a> Chimper<'a> {
  fn new(logoid: conrod::image::Id, imap: &'a Mutex<ImageState>,) -> Self {
    Self {
      dragwidth: 5.0,
      sidewidth: 600.0,
      use_sidepane: true,
      imagepadding: 20.0,
      logoid,
      ids: None,
      imap,
      file: None,
      directory: env::current_dir().unwrap(),
      sideopt: true,
      orientation: 0,
    }
  }
}

#[derive(Debug, Clone)]
struct DisplayableImage {
  id: conrod::image::Id,
  width: u32,
  height: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct RequestedImage {
  file: String,
  size: usize,
}

#[derive(Debug, Clone)]
enum ImageState {
  NoneSelected,
  Requested{request: RequestedImage, current: Option<DisplayableImage>},
  Loaded{request: RequestedImage, current: DisplayableImage},
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
    let dragwidth = self.dragwidth * ((self.use_sidepane as u8) as f64);
    {
      let ui = &mut ui.set_widgets();

      // Construct our main `Canvas` tree.
      widget::Canvas::new().flow_right(&[
        (ids.imgcanvas, widget::Canvas::new().color(color::CHARCOAL).border(0.0)),
        (ids.dragcanvas, widget::Canvas::new().length(dragwidth).color(color::BLACK).border(0.0)),
        (ids.setcanvas, widget::Canvas::new().length(sidewidth).border(0.0).flow_down(&[
          (ids.settop, widget::Canvas::new().color(color::GREY).length(100.0).border(0.0)),
          (ids.setcont, widget::Canvas::new().color(color::GREY).border(0.0)),
        ])),
      ]).border(0.0).set(ids.background, ui);

      let size = chimper::cache::smallest_size(ui.win_w as usize, ui.win_h as usize);

      let new_state = {
        let imap = self.imap.lock().unwrap();

        let (new_state, image) = match self.file {
          None => (None, None),
          Some(ref file) => {
            let new_request = RequestedImage {
              file: (*file).clone(),
              size: size,
            };
            match *imap {
              ImageState::NoneSelected => {
                (Some(ImageState::Requested {request: new_request, current: None}), None)
              },
              ImageState::Requested{ref request, ref current} => {
                if new_request != *request {
                  (Some(ImageState::Requested{request: new_request, current: current.clone()}), current.clone())
                } else {
                  (None, current.clone())
                }
              },
              ImageState::Loaded{ref request, ref current} => {
                if new_request != *request {
                  (Some(ImageState::Requested{request: new_request, current: Some(current.clone())}), Some(current.clone()))
                } else {
                  (None, Some(current.clone()))
                }
              },
            }
          }
        };

        if let Some(image) = image {
          let scale = (image.width as f64)/(image.height as f64);
          let mut width = (ui.w_of(ids.imgcanvas).unwrap() - self.imagepadding).min(image.width as f64);
          let mut height = (ui.h_of(ids.imgcanvas).unwrap() - self.imagepadding).min(image.height as f64);
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
        new_state
      };

      if let Some(new_state) = new_state {
        let mut imap = self.imap.lock().unwrap();
        *imap = new_state;
        evproxy.wakeup().is_ok();
      }

      if sidewidth > 0.0 {
        for _event in widget::Button::image(self.logoid)
          .w_h(78.0, 88.0)
          .top_right_with_margin_on(ids.settop, 6.0)
          .set(ids.chimper, ui) 
        {
          self.sideopt = !self.sideopt;
        }

        if self.sideopt {
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
        } else {
          for event in widget::drop_down_list::DropDownList::new(&ORIENTATION_NAMES, Some(self.orientation))
            .w_h(130.0, 30.0)
            .top_left_with_margin_on(ids.setcont, 6.0)
            .set(ids.dropdown, ui)
          {
            self.orientation = event;
          }
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
  let imap = Mutex::new(ImageState::NoneSelected);

  crossbeam_utils::thread::scope(|scope| {
    let mut chimp = Chimper::new(logoid, &imap);
    window.run(&mut chimp, |display, _rederer, image_map, evproxy| {
      //Load images if needed
      let image = {
        match *(imap.lock().unwrap()) {
          ImageState::NoneSelected |
          ImageState::Loaded{..} => None, // There's nothing to do
          ImageState::Requested{ref request, ..} => Some((request.clone(), icache.get(request.file.clone(), request.size, scope, evproxy))),
        }
      };

      if let Some((request, image)) = image {
        if let Some(ref imgbuf) = *image {
          // We've finished a request and need to pass it on to be displayed

          // Save the old id for later removal
          let oldid = match *(imap.lock().unwrap()) {
            ImageState::NoneSelected |
            ImageState::Loaded{..} => panic!("I have a new image and yet am not in a Requested state!"),
            ImageState::Requested{ref current, ..} => {
              if let Some(current) = current {
                Some(current.id)
              } else {
                None
              }
            },
          };

          // Create a new image
          let dims = (imgbuf.width as u32, imgbuf.height as u32);
          let raw_image = glium::texture::RawImage2d::from_raw_rgb_reversed(&imgbuf.data, dims);
          let img = glium::texture::SrgbTexture2d::with_format(
            display,
            raw_image,
            glium::texture::SrgbFormat::U8U8U8,
            glium::texture::MipmapsOption::NoMipmap
          ).unwrap();
          let newid = image_map.insert(img);
          let newimage = DisplayableImage {
            id: newid,
            width: dims.0,
            height: dims.1,
          };

          { // Set the new state so from now on draws use this image
            let mut imap = imap.lock().unwrap();
            *imap = ImageState::Loaded{request: request.clone(), current: newimage};
          }

          // Cleanup the old image, now unused
          if let Some(oldid) = oldid {
            image_map.remove(oldid);
          }

          true // cause a redraw
        } else {
          false
        }
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
