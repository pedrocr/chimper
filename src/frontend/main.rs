extern crate imagepipe;
extern crate conrod_core;
extern crate glium;

use std::env;
use std::sync::Mutex;
use std::path::PathBuf;
extern crate crossbeam_utils;
extern crate image;

use crate::frontend::*;
use crate::backend::cache;
use crate::backend::cache::RequestedImage;

widget_ids!(
pub struct ChimperIds {
  background, imgcanvas, dragcanvas, setcanvas, settop, setcont, raw_image, chimper, filenav,
  ops_settings[],
  ops_headers[],

  op_rawinput[],
  op_tolab[],
  op_transform[],
  op_basecurve[],
});

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SelectedOp {
  None,
  RawInput,
  Level,
  ToLab,
  Basecurve,
  Transform,
}

pub struct Chimper<'a> {
  pub dragwidth: f64,
  pub sidewidth: f64,
  pub imagepadding: f64,
  pub use_sidepane: bool,
  pub logoid: conrod_core::image::Id,
  pub ids: Option<ChimperIds>,
  pub sideopt: bool,
  pub directory: std::path::PathBuf,
  pub file: Option<String>,
  pub image: Option<DisplayableImage>,
  pub ops: Mutex<Option<(String, imagepipe::PipelineOps)>>,
  pub selected_op: SelectedOp,
  pub fullscreen: &'a Mutex<bool>,

  imap: &'a Mutex<ImageState>,
}

impl<'a> Chimper<'a> {
  fn new(logoid: conrod_core::image::Id, imap: &'a Mutex<ImageState>, fullscreen: &'a Mutex<bool>, path: Option<PathBuf>) -> Self {

    let (file, directory, sideopt) = if let Some(path) = path {
      if path.is_file() {
        let file = path.to_str().unwrap().to_string();
        let directory = path.parent().unwrap().to_owned();
        (Some(file), directory, false)
      } else {
        (None, path, true)
      }
    } else {
      (None, env::current_dir().unwrap(), true)
    };

    Self {
      dragwidth: 5.0,
      sidewidth: 600.0,
      use_sidepane: true,
      imagepadding: 20.0,
      logoid,
      ids: None,
      imap,
      file,
      directory,
      sideopt,
      ops: Mutex::new(None),
      image: None,
      selected_op: SelectedOp::None,
      fullscreen,
    }
  }
}

#[derive(Debug, Clone)]
pub struct DisplayableImage {
  pub id: conrod_core::image::Id,
  pub width: u32,
  pub height: u32,
}

#[derive(Debug, Clone)]
enum ImageState {
  NoneSelected,
  Requested{request: RequestedImage, current: Option<DisplayableImage>},
  Loaded{request: RequestedImage, current: DisplayableImage},
}

impl<'a> window::ChimperApp for Chimper<'a> {
  fn initialize(&mut self, ui: &mut conrod_core::Ui) {
    self.ids = Some(ChimperIds::new(ui.widget_id_generator()));
  }

  fn draw_gui(&mut self, ui: &mut conrod_core::Ui, evproxy: &glium::glutin::EventsLoopProxy) -> bool {
    {
      // While we're drawing the UI the request mutex is ours
      let mut imap = self.imap.lock().unwrap();
      let mut currops = self.ops.lock().unwrap();

      if let Some(ref file) = self.file {
        let opsfile = match *currops {
          Some((ref f, _)) => Some(f.clone()),
          None => None,
        };

        if let Some(opsfile) = opsfile {
          if &opsfile != file {
            // We already had ops but they're for an old file, try and get new ones and if it fails
            // set empty
            *currops = if let ImageState::Loaded{ref request, ..} = *imap {
              if let Some(ref ops) = request.ops {
                if &(request.file) == file {
                  // We swap our ops for the new file
                  Some((file.clone(), ops.clone()))
                } else { None }
              } else { None }
            } else { None };
          }
        } else {
          // Try and get ops for this new file, if that fails it's already empty
          if let ImageState::Loaded{ref request, ..} = *imap {
            if let Some(ref ops) = request.ops {
              if &(request.file) == file {
                *currops = Some((file.clone(), ops.clone()));
              }
            }
          }
        }
      } else {
        // We don't have a file so we should't have ops either
        *currops = None;
      }

      let size = cache::smallest_size(ui.win_w as usize, ui.win_h as usize);
      let ops = if let Some((_, ref ops)) = *currops {
        Some(ops.clone())
      } else {
        None
      };

      let (new_state, image) = match self.file {
        None => (None, None),
        Some(ref file) => {
          let new_request = RequestedImage {
            file: (*file).clone(),
            size: size,
            ops: ops,
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
      self.image = image;

      if let Some(new_state) = new_state {
        *imap = new_state;
        evproxy.wakeup().unwrap();
      }
    }

    // Now that we have our state in order and our requests out of the day lets draw ou UI and
    // process any events
    gui::draw_gui(self, ui)
  }

  fn process_event(&mut self, event: &conrod_core::event::Input) {
    match *event {
      conrod_core::event::Input::Press(conrod_core::input::Button::Keyboard(conrod_core::input::Key::Tab)) => {
        self.use_sidepane = !self.use_sidepane;
      },
      _ => (),
    }
  }
}


pub fn run_app(path: Option<PathBuf>) {
  let mut window = window::ChimperWindow::new("Chimper", 1200, 800);
  let logoid = window.load_texture(load_image(logo::random()));

  let icache = cache::ImageCache::new();
  let imap = Mutex::new(ImageState::NoneSelected);
  let fullscreen = Mutex::new(false);
  let oldids: Mutex<Vec<(conrod_core::image::Id, u64)>> = Mutex::new(Vec::new());
  let oldsref = &oldids;

  crossbeam_utils::thread::scope(|scope| {
    let mut chimp = Chimper::new(logoid, &imap, &fullscreen, path);

    window.run(&mut chimp, |display, _rederer, image_map, evproxy, frame_count, fs| {
      {
        *(fullscreen.lock().unwrap()) = fs;
      }

      // Remove old images if we're already displaying a following frame and thus there are no
      // more remaining references to them
      let mut oldids = oldsref.lock().unwrap();
      (*oldids).retain(|&(id, frame)| {
        if frame_count > frame {
          image_map.remove(id);
          false
        } else {
          true
        }
      });

      //Load images if needed
      let mut imap = imap.lock().unwrap();
      let image = {
        match *imap {
          ImageState::NoneSelected |
          ImageState::Loaded{..} => None, // There's nothing to do
          ImageState::Requested{ref request, ref current} => Some((
            request.clone(),
            current.clone(),
            icache.get(request.clone(), scope, evproxy)
          )),
        }
      };

      let mut needs_redraw = false;
      if let Some((request, current, image)) = image {
        if let Some(ref image) = *image {
          // We've finished a request and need to pass it on to be displayed
          let (imgbuf, ops) = image;

          // Save the old id for later removal
          if let Some(current) = current {
            oldids.push((current.id, frame_count));
          }

          // Create a new image
          let dims = (imgbuf.width as u32, imgbuf.height as u32);
          let raw_image = glium::texture::RawImage2d::from_raw_rgb_reversed(&imgbuf.data, dims);
          let img = glium::texture::SrgbTexture2d::with_format(
            display,
            raw_image,
            glium::texture::SrgbFormat::U8U8U8,
            glium::texture::MipmapsOption::NoMipmap
          ).unwrap();
          let newimage = DisplayableImage {
            id: image_map.insert(img),
            width: dims.0,
            height: dims.1,
          };

          // Set the new state so from now on draws use this image
          let mut filled_request = request.clone();
          filled_request.ops = Some(ops.clone());
          *imap = ImageState::Loaded{request: filled_request, current: newimage};

          needs_redraw = true;
        }
      }
      needs_redraw
    });
  }).unwrap();
}

// Load the image from a file
pub fn load_image(buf: &[u8]) -> glium::texture::RawImage2d<u8> {
  let img = image::load_from_memory(buf).unwrap().to_rgba8();
  let dims = img.dimensions();
  glium::texture::RawImage2d::from_raw_rgba_reversed(&img.into_raw(), dims)
}
