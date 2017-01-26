#[macro_use] extern crate conrod;
use conrod::{widget, Colorable, Positionable, Sizeable, Borderable, Widget, color};
use conrod::backend::piston::gfx::{GfxContext, G2dTexture, Texture, TextureSettings};
use conrod::backend::piston::{self, Window, WindowEvents};
use conrod::backend::piston::draw::ImageSize;
use conrod::backend::piston::event::UpdateEvent;

extern crate gfx_device_gl;

use std::env;
#[macro_use] extern crate lazy_static;
extern crate crossbeam;
extern crate image;

mod cache;
mod logo;

fn main() {
  let args: Vec<_> = env::args().collect();
  if args.len() != 2 {
    println!("Usage: {} <file>", args[0]);
    std::process::exit(2);
  }
  let file = &args[1];
  println!("Loading file \"{}\"", file);

  const WIDTH: u32 = 1200;
  const HEIGHT: u32 = 800;

  // Construct the window.
  let mut window: Window =
    piston::window::WindowSettings::new("Chimper", [WIDTH, HEIGHT])
      .exit_on_esc(true).vsync(true).build().unwrap();

  // Create the event loop.
  let mut events = WindowEvents::new();

  // construct our `Ui`.
  let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

  // Create an empty texture to pass for the text cache as we're not drawing any text.
  let mut text_texture_cache = piston::window::GlyphCache::new(&mut window, 0, 0);

  // The `WidgetId` for our background and `Image` widgets.
  widget_ids!(struct Ids { background, imgcanvas, setcanvas, raw_image, chimper });
  let ids = Ids::new(ui.widget_id_generator());

  let mut image_map = image_map! {
      (ids.chimper, load_image(logo::random(), &mut window.context)),
  };

  let mut currsize = 0 as usize; // Initially set the image size to smallest
  let icache = cache::ImageCache::new();
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
      let size = icache.smallest_size(width as usize, height as usize);

      if image_map.get(&ids.raw_image).is_none() || size != currsize {
        match icache.get(&file, size, scope) {
          None => {},
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
          width = (width-620.0).min(maxw as f64);
          height = (height-20.0).min(maxh as f64);
          if width/height > scale {
            width = height * scale;
          } else {
            height = width / scale;
          }
        },
      }

      event.update(|_| {
        let ui = &mut ui.set_widgets();

        // Construct our main `Canvas` tree.
        widget::Canvas::new().flow_right(&[
            (ids.imgcanvas, widget::Canvas::new().color(color::CHARCOAL).border(0.0)),
            (ids.setcanvas, widget::Canvas::new().color(color::GREY).length(600.0).border(0.0)),
        ]).border(0.0).set(ids.background, ui);

        widget::Image::new().w_h(width, height).top_left().middle_of(ids.imgcanvas)
          .set(ids.raw_image, ui);
        widget::Image::new().w_h(78.0, 88.0).top_right_with_margin_on(ids.setcanvas, 6.0)
          .set(ids.chimper, ui);
      });
    }
  });
}

// Load the image from a file
fn load_image(buf: &[u8], context: &mut GfxContext) -> G2dTexture<'static> {
  let img = image::load_from_memory(buf).unwrap().to_rgba();
  let factory = &mut context.factory;
  let settings = TextureSettings::new();
  Texture::from_image(factory, &img, &settings).unwrap()
}
