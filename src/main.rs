#[macro_use] extern crate conrod;
use conrod::{widget, Colorable, Positionable, Sizeable, Widget, color};
use conrod::backend::piston::gfx::{GfxContext, G2dTexture, Texture, TextureSettings};
use conrod::backend::piston::{self, Window, WindowEvents, OpenGL};
use conrod::backend::piston::draw::ImageSize;
use conrod::backend::piston::event::UpdateEvent;

extern crate gfx_device_gl;

extern crate image;
use image::{ImageBuffer, Rgba};

extern crate rawloader;
use std::env;
use std::sync::RwLock;
use std::collections::HashMap;
#[macro_use] extern crate lazy_static;

extern crate crossbeam;

extern crate rand;
use rand::distributions::{IndependentSample, Range};

lazy_static! {
  static ref ILOCK: RwLock<HashMap<(String, usize), ImageBuffer<Rgba<u8>, Vec<u8>>>> = RwLock::new(HashMap::new());
}

const SIZES: [[usize;2];7] = [
  [640, 480],   //  0,3MP - Small thumbnail
  [1400, 800],  //  1,1MP - 720p+
  [2000, 1200], //  2,4MP - 1080p+
  [2600, 1600], //  4,2MP - WQXGA
  [4100, 2200], //  9,0MP - 4K
  [5200, 2900], // 15,1MP - 5K
  [0, 0],       // Go full size above 5K
];

fn smallest_size(width: usize, height: usize) -> usize {
  for (i,vals) in SIZES.iter().enumerate() {
    if vals[0] >= width && vals[1] >= height {
      return i
    }
  }
  return SIZES.len() - 1
}

fn main() {
  let args: Vec<_> = env::args().collect();
  if args.len() != 2 {
    println!("Usage: {} <file>", args[0]);
    std::process::exit(2);
  }
  let file = &args[1];
  println!("Loading file \"{}\"", file);

  const WIDTH: u32 = 800;
  const HEIGHT: u32 = 600;

  // Change this to OpenGL::V2_1 if not working.
  let opengl = OpenGL::V3_2;

  // Construct the window.
  let mut window: Window =
    piston::window::WindowSettings::new("Chimper", [WIDTH, HEIGHT])
      .opengl(opengl).exit_on_esc(true).vsync(true).samples(4).build().unwrap();

  // Create the event loop.
  let mut events = WindowEvents::new();

  // construct our `Ui`.
  let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

  // Create an empty texture to pass for the text cache as we're not drawing any text.
  let mut text_texture_cache = piston::window::GlyphCache::new(&mut window, 0, 0);

  // The `WidgetId` for our background and `Image` widgets.
  widget_ids!(struct Ids { background, raw_image, chimper });
  let ids = Ids::new(ui.widget_id_generator());

  let mut logos = Vec::<&'static [u8]>::new();
  logos.push(include_bytes!("../icons/chimp1.svg.png"));
  logos.push(include_bytes!("../icons/chimp2.svg.png"));
  logos.push(include_bytes!("../icons/chimp3.svg.png"));
  logos.push(include_bytes!("../icons/chimp4.svg.png"));
  logos.push(include_bytes!("../icons/chimp5.svg.png"));
  logos.push(include_bytes!("../icons/chimp6.svg.png"));
  logos.push(include_bytes!("../icons/chimp7.svg.png"));
  logos.push(include_bytes!("../icons/chimp8.svg.png"));
  logos.push(include_bytes!("../icons/chimp9.svg.png"));

  let between = Range::new(0, logos.len());
  let mut rng = rand::thread_rng();
  let idx = between.ind_sample(&mut rng);

  let mut image_map = image_map! {
      (ids.chimper, load_image(logos[idx], &mut window.context)),
  };

  let mut currsize = 0 as usize; // Initially set the image size to smallest
  let mut changesize = 1000 as usize; // When we start we are not changing to any other size
  crossbeam::scope(|scope| {
    // Poll events from the window.
    while let Some(event) = window.next_event(&mut events) {
      // Convert the piston event to a conrod input event.
      if let Some(e) = piston::window::convert_event(event.clone(), &window) {
          ui.handle_event(e);
      }

      window.draw_2d(&event, |c, g| {
        if let Some(primitives) = ui.draw_if_changed() {
          fn texture_from_image<T>(img: &T) -> &T { img };
          piston::window::draw(c, g, primitives,
                                               &mut text_texture_cache,
                                               &image_map,
                                               texture_from_image);
        }
      });

      let (width,height) = window.window.window.get_inner_size_pixels().unwrap();
      let size = smallest_size(width as usize, height as usize);

      if image_map.get(&ids.raw_image).is_none() || size != currsize {
        let image_cache = ILOCK.read().unwrap();
        match image_cache.get(&(file.clone(), size)) {
          None => {
            if size != changesize {
              // First time we have found the change launch the thread to load the image
              load_raw(&file, size, scope);
            }
            changesize = size;
          },
          Some(imgbuf) => {
            let settings = TextureSettings::new();
            let factory = &mut window.context.factory;
            let img = Texture::from_image(factory, &imgbuf, &settings).unwrap();
            image_map.insert(ids.raw_image, img);
            currsize = size;
          },
        }
      }

      let mut width = width as f64;
      let mut height = height as f64;
      match image_map.get(&ids.raw_image) {
        None => {},
        Some(img) => {
          let (maxw, maxh) = img.get_size();
          let maxw = maxw;
          let scale = (maxw as f64)/(maxh as f64);
          width = (width-90.0).min(maxw as f64);
          height = height.min(maxh as f64);
          if width/height > scale {
            width = height * scale;
          } else {
            height = width / scale;
          }
        },
      }

      event.update(|_| {
        let ui = &mut ui.set_widgets();
        widget::Canvas::new().color(color::CHARCOAL).set(ids.background, ui);
        widget::Image::new().w_h(width, height).top_left().set(ids.raw_image, ui);
        widget::Image::new().w_h(78.0, 88.0).top_right_with_margin(6.0).set(ids.chimper, ui);
      });
    }
  });
}

// Load the image from a file
fn load_raw<'a>(path: &'a str, size: usize, scope: &crossbeam::Scope<'a>) {
  let file = path.to_string();
  let maxwidth = SIZES[size][0];
  let maxheight = SIZES[size][1];

  scope.spawn(move || {
    let decoded = rawloader::decode(path).unwrap().to_rgb(maxwidth, maxheight).unwrap();
    // Convert f32 RGB into u8 RGBA
    let mut buffer = vec![0 as u8; (decoded.width*decoded.height*4) as usize];
    for (pixin, pixout) in decoded.data.chunks(3).zip(buffer.chunks_mut(4)) {
      pixout[0] = (pixin[0]*255.0).max(0.0).min(255.0) as u8;
      pixout[1] = (pixin[1]*255.0).max(0.0).min(255.0) as u8;
      pixout[2] = (pixin[2]*255.0).max(0.0).min(255.0) as u8;
      pixout[3] = 255;
    }
    let img = ImageBuffer::from_raw(decoded.width as u32, decoded.height as u32, buffer).unwrap();
    let mut image_cache = ILOCK.write().unwrap();
    image_cache.insert((file, size), img);
  });
}

// Load the image from a file
fn load_image(buf: &[u8], context: &mut GfxContext) -> G2dTexture<'static> {
  let img = image::load_from_memory(buf).unwrap().to_rgba();
  let factory = &mut context.factory;
  let settings = TextureSettings::new();
  Texture::from_image(factory, &img, &settings).unwrap()
}
