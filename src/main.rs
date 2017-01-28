#[macro_use] extern crate conrod;
use conrod::{widget, Colorable, Positionable, Sizeable, Borderable, Widget, color};
use conrod::backend::glium::glium;
use conrod::backend::glium::glium::{DisplayBuild, Surface};

extern crate gfx_device_gl;

use std::env;
#[macro_use] extern crate lazy_static;
extern crate crossbeam;
extern crate image;

mod cache;
mod logo;
mod event;

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

  // Build the window.
  let display = glium::glutin::WindowBuilder::new()
    .with_vsync()
    .with_dimensions(WIDTH, HEIGHT)
    .with_title("Chimper")
    .build_glium()
    .unwrap();

  let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

  // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
  // for drawing to the glium `Surface`.
  let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

  // The `WidgetId` for our background and `Image` widgets.
  widget_ids!(struct Ids { background, imgcanvas, setcanvas, raw_image, chimper });
  let ids = Ids::new(ui.widget_id_generator());

  let mut image_map = conrod::image::Map::new();
  let logoid = image_map.insert(load_image(logo::random(), &display));
  let mut rawid: Option<conrod::image::Id> = None;

  let mut currsize = 1000 as usize; // Initially set the image size to an impossible size
  let sidewidth = 600.0;
  let mut use_sidepane = true;
  let imagepadding = 20.0;
  let icache = cache::ImageCache::new();
  let context = event::UIContext::new();
  crossbeam::scope(|scope| {
    // Poll events from the window.
    'main: loop {
      // Handle all events.
      for event in context.next(&display) {
        // Use the `winit` backend feature to convert the winit event to a conrod one.
        if let Some(event) = conrod::backend::winit::convert(event.clone(), &display) {
            ui.handle_event(event);
        }

        match event {
          // Break from the loop upon `Escape`.
          glium::glutin::Event::KeyboardInput(_, _, Some(glium::glutin::VirtualKeyCode::Escape)) |
          glium::glutin::Event::Closed =>
            break 'main,
          glium::glutin::Event::KeyboardInput(glium::glutin::ElementState::Pressed, _, Some(glium::glutin::VirtualKeyCode::Tab)) => {
            use_sidepane = !use_sidepane;
          }

          _ => {},
        }
      }

      let (width,height) = display.get_window().unwrap().get_inner_size_pixels().unwrap();
      let size = icache.smallest_size(width as usize, height as usize);

      if size != currsize {
        match icache.get(&file, size, scope, &context) {
          None => {},
          Some(imgbuf) => {
            let dims = imgbuf.dimensions();
            let img = (*imgbuf).clone().into_raw();
            let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(img, dims);
            let img = glium::texture::Texture2d::new(&display, raw_image).unwrap();
            rawid = Some(image_map.insert(img));
            currsize = size;
          },
        }
      }

      let sidewidth = sidewidth * ((use_sidepane as u8) as f64);
      {
        let ui = &mut ui.set_widgets();

        // Construct our main `Canvas` tree.
        widget::Canvas::new().flow_right(&[
            (ids.imgcanvas, widget::Canvas::new().color(color::CHARCOAL).border(0.0)),
            (ids.setcanvas, widget::Canvas::new().color(color::GREY).length(sidewidth).border(0.0)),
        ]).border(0.0).set(ids.background, ui);

        if let Some(rawid) = rawid {
          let mut width = width as f64;
          let mut height = height as f64;
          match image_map.get(&rawid) {
            None => {},
            Some(img) => {
              let (maxw, maxh) = img.dimensions();
              let scale = (maxw as f64)/(maxh as f64);
              width = (width-sidewidth-imagepadding).min(maxw as f64);
              height = (height-imagepadding).min(maxh as f64);
              if width/height > scale {
                width = height * scale;
              } else {
                height = width / scale;
              }
            },
          }

          widget::Image::new(rawid)
            .w_h(width, height)
            .middle_of(ids.imgcanvas)
            .set(ids.raw_image, ui);
        }
        if sidewidth > 0.0 {
          widget::Image::new(logoid)
            .w_h(78.0, 88.0)
            .top_right_with_margin_on(ids.setcanvas, 6.0)
            .set(ids.chimper, ui);
        }
      }

      // Render the `Ui` and then display it on the screen.
      if let Some(primitives) = ui.draw_if_changed() {
        renderer.fill(&display, primitives, &image_map);
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        renderer.draw(&display, &mut target, &image_map).unwrap();
        target.finish().unwrap();
      }
    }
  });
}

// Load the image from a file
fn load_image(buf: &[u8], display: &glium::Display) -> glium::texture::Texture2d {
  let img = image::load_from_memory(buf).unwrap().to_rgba();
  let dims = img.dimensions();
  let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(img.into_raw(), dims);
  glium::texture::Texture2d::new(display, raw_image).unwrap()
}
