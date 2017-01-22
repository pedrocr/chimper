#[macro_use] extern crate conrod;
use conrod::{widget, Colorable, Positionable, Sizeable, Widget, color};
use conrod::backend::piston::gfx::{G2dTexture, Texture, TextureSettings};
use conrod::backend::piston::{self, Window, WindowEvents, OpenGL};
use conrod::backend::piston::draw::ImageSize;
use conrod::backend::piston::event::UpdateEvent;

extern crate gfx_device_gl;

extern crate image;
use image::ImageBuffer;

extern crate rawloader;
use std::env;
use std::thread;
use std::sync::RwLock;

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
  widget_ids!(struct Ids { background, raw_image });
  let ids = Ids::new(ui.widget_id_generator());

  let wlock = RwLock::new(window);
  let ilock = RwLock::new(conrod::image::Map::new());
  //thread::spawn(||
  {
    let mut image_map = ilock.write().unwrap();
    image_map.insert(ids.raw_image, load_image(&wlock, &file));
  }//);

  // Poll events from the window.
  loop {
    let mut window = wlock.write().unwrap();
    match (*window).next_event(&mut events) {
      None => break,
      Some(event) => {
        // Convert the piston event to a conrod input event.
        if let Some(e) = piston::window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        let image_map = ilock.read().unwrap();

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
        let mut width = width as f64;
        let mut height = height as f64;

        match image_map.get(&ids.raw_image) {
          None => {},
          Some(img) => {
            let (maxw, maxh) = img.get_size();
            let scale = (maxw as f64)/(maxh as f64);
            width = width.min(maxw as f64);
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
          // Draw a light blue background.
          widget::Canvas::new().color(color::BLACK).set(ids.background, ui);
          // Instantiate the `Image` at its full size in the middle of the window.
          widget::Image::new().w_h(width, height).middle().set(ids.raw_image, ui);
        });
      }
    }
  }
}

// Load the image from a file
fn load_image(wlock: &RwLock<Window>, path: &str) -> G2dTexture<'static> {
  let decoded = rawloader::decode(path).unwrap().to_rgb(0, 0).unwrap();

  // Convert f32 RGB into u8 RGBA
  let mut buffer = vec![0 as u8; (decoded.width*decoded.height*4) as usize];
  for (pixin, pixout) in decoded.data.chunks(3).zip(buffer.chunks_mut(4)) {
    pixout[0] = (pixin[0]*255.0).max(0.0).min(255.0) as u8;
    pixout[1] = (pixin[1]*255.0).max(0.0).min(255.0) as u8;
    pixout[2] = (pixin[2]*255.0).max(0.0).min(255.0) as u8;
    pixout[3] = 255;
  }  
  let imgbuf = ImageBuffer::from_raw(decoded.width as u32, decoded.height as u32, buffer).unwrap();

  let settings = TextureSettings::new();
  let mut window = wlock.write().unwrap();
  let factory = &mut window.context.factory;
  Texture::from_image(factory, &imgbuf, &settings).unwrap()
}
