#[macro_use] extern crate conrod;
extern crate piston_window;
extern crate image;
use conrod::{widget, Colorable, Positionable, Sizeable, Widget, color};
use piston_window::{EventLoop, ImageSize, G2dTexture, PistonWindow, Texture, UpdateEvent, Window};
use image::ImageBuffer;

extern crate rawloader;
use rawloader::decoders;
use rawloader::imageops;
use std::env;
use std::cmp;

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
  let opengl = piston_window::OpenGL::V3_2;

  // Construct the window.
  let mut window: PistonWindow =
    piston_window::WindowSettings::new("Chimper", [WIDTH, HEIGHT])
      .opengl(opengl).exit_on_esc(true).vsync(true).samples(4).build().unwrap();
  window.set_ups(60);

  // construct our `Ui`.
  let mut ui = conrod::UiBuilder::new().build();

  // Create an empty texture to pass for the text cache as we're not drawing any text.
  let mut text_texture_cache = conrod::backend::piston_window::GlyphCache::new(&mut window, 0, 0);

  // The `WidgetId` for our background and `Image` widgets.
  widget_ids!(struct Ids { background, raw_image });
  let ids = Ids::new(ui.widget_id_generator());

  // Create our `conrod::image::Map` which describes each of our widget->image mappings.
  // In our case we only have one image, however the macro may be used to list multiple.
  let image_map = image_map! {
    (ids.raw_image, load_image(&mut window, &file)),
  };

  // We'll instantiate the `Image` at its full size, so we'll retrieve its dimensions.
  let (maxw, maxh) = image_map.get(&ids.raw_image).unwrap().get_size();
  let scale = (maxw as f64)/(maxh as f64);

  // Poll events from the window.
  while let Some(event) = window.next() {
    ui.handle_event(event.clone());

    window.draw_2d(&event, |c, g| {
      if let Some(primitives) = ui.draw_if_changed() {
        fn texture_from_image<T>(img: &T) -> &T { img };
        conrod::backend::piston_window::draw(c, g, primitives,
                                             &mut text_texture_cache,
                                             &image_map,
                                             texture_from_image);
      }
    });

    let wsize = window.window.draw_size();
    let mut width = cmp::min(wsize.width, maxw) as f64;
    let mut height = cmp::min(wsize.height, maxh) as f64;
    if width/height > scale {
      width = height * scale;
    } else {
      height = width / scale;
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

// Load the image from a file
fn load_image(window: &mut PistonWindow, path: &str) -> G2dTexture<'static> {
  let rawloader = decoders::RawLoader::new();
  let image = rawloader.decode_safe(path).unwrap();
  let decoded = imageops::simple_decode(&image);  

  // Convert f32 RGB into u8 RGBA
  let mut buffer = vec![0 as u8; (image.width*image.height*4) as usize];
  for (pixin, pixout) in decoded.chunks(3).zip(buffer.chunks_mut(4)) {
    pixout[0] = (pixin[0]*255.0).max(0.0).min(255.0) as u8;
    pixout[1] = (pixin[1]*255.0).max(0.0).min(255.0) as u8;
    pixout[2] = (pixin[2]*255.0).max(0.0).min(255.0) as u8;
    pixout[3] = 255;
  }  
  let imgbuf = ImageBuffer::from_raw(image.width as u32, image.height as u32, buffer).unwrap();

  let factory = &mut window.factory;
  let settings = piston_window::TextureSettings::new();
  Texture::from_image(factory, &imgbuf, &settings).unwrap()
}
