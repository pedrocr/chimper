#[macro_use] extern crate conrod;
use conrod::{widget, Colorable, Positionable, Sizeable, Borderable, Widget, color};
use conrod::backend::glium::glium::{self, Surface};
use conrod::backend::glium::glium::texture::srgb_texture2d::SrgbTexture2d;
use conrod::backend::glium::glium::glutin::{Event, WindowEvent, VirtualKeyCode};
use std::env;
use std::sync::{Arc, Mutex};
extern crate crossbeam;
extern crate image;
extern crate rusttype;

mod cache;
mod logo;

struct ImageMapping {
  id: Option<(String, usize)>,
  img: Option<(conrod::image::Id,u32,u32)>,
  alldone: bool,
}

fn main() {
  const WIDTH: u32 = 1200;
  const HEIGHT: u32 = 800;
  //let mut fullscreen = false;

  // Build the window.
  let mut evloop = glium::glutin::EventsLoop::new();
  let window = glium::glutin::WindowBuilder::new()
    .with_title("Chimper")
    .with_dimensions(WIDTH, HEIGHT);
  let context = glium::glutin::ContextBuilder::new()
    .with_vsync(true)
    .with_multisampling(4);
  let display = glium::Display::new(window, context, &evloop).unwrap();

  // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
  // for drawing to the glium `Surface`.
  let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

  let mut image_map = conrod::image::Map::new();
  let logoid = image_map.insert(load_image(logo::random(), &display));

  // A channel to send events from the main `winit` thread to the conrod thread.
  let (event_tx, event_rx) = std::sync::mpsc::channel();
  // A channel to send `render::Primitive`s from the conrod thread to the `winit thread.
  let (render_tx, render_rx) = std::sync::mpsc::channel();

  // The main conrod UI loop
  fn run_conrod(logoid: conrod::image::Id,
                imap: &Mutex<ImageMapping>,
                event_rx: std::sync::mpsc::Receiver<conrod::event::Input>,
                render_tx: std::sync::mpsc::Sender<conrod::render::OwnedPrimitives>,
                evproxy: glium::glutin::EventsLoopProxy) {
    let mut file: Option<String> = None;
    let currdir = env::current_dir().unwrap();
    let directory = currdir.as_path();

    // Construct our `Ui`.
    let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();
    ui.fonts.insert(load_font(include_bytes!("../fonts/NotoSans-Regular.ttf")));

    // The `WidgetId` for our background and `Image` widgets.
    widget_ids!(struct Ids { background, imgcanvas, dragcanvas, setcanvas, settop, setcont, raw_image, chimper, filenav });
    let ids = Ids::new(ui.widget_id_generator());

    let dragwidth = 10.0;
    let sidewidth = 600.0;
    let mut use_sidepane = true;
    let imagepadding = 20.0;

    // Many widgets require another frame to finish drawing after clicks or hovers, so we
    // insert an update into the conrod loop using this `bool` after each event.
    let mut needs_update = true;
    'conrod: loop {
      // Collect any pending events.
      let mut events = Vec::new();
      while let Ok(event) = event_rx.try_recv() {
        events.push(event);
      }

      // If there are no events pending, wait for them.
      if events.is_empty() || !needs_update {
        match event_rx.recv() {
          Ok(event) => events.push(event),
          Err(_) => break 'conrod,
        };
      }

      needs_update = false;
      // Input each event into the `Ui`.
      for event in events {
        match event {
          conrod::event::Input::Press(conrod::input::Button::Keyboard(conrod::input::Key::Tab)) => {
            use_sidepane = !use_sidepane;
          },
          _ => (),
        }
        ui.handle_event(event);
        needs_update = true;
      }

      let sidewidth = sidewidth * ((use_sidepane as u8) as f64);
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

        let size = cache::smallest_size(ui.win_w as usize, ui.win_h as usize);
        let img = {
          match file {
            None => None,
            Some(ref file) => {
              let newid = Some(((*file).clone(), size));
              let mut imap = imap.lock().unwrap();
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
          let mut width = (ui.w_of(ids.imgcanvas).unwrap() - imagepadding).min(maxw as f64);
          let mut height = (ui.h_of(ids.imgcanvas).unwrap() - imagepadding).min(maxh as f64);
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
          widget::Image::new(logoid)
            .w_h(78.0, 88.0)
            .top_right_with_margin_on(ids.settop, 6.0)
            .set(ids.chimper, ui);
        }

        for event in widget::FileNavigator::all(&directory)
          .color(conrod::color::LIGHT_BLUE)
          .font_size(16)
          .kid_area_wh_of(ids.setcont)
          .middle_of(ids.setcont)
          //.show_hidden_files(true)  // Use this to show hidden files
          .set(ids.filenav, ui)
        {
          //println!("Caught event {:?}", event);
          match event {
            conrod::widget::file_navigator::Event::ChangeSelection(pbuf) => {
              if pbuf.len() > 0 {
                let path = pbuf[0].as_path();
                if path.is_file() {
                  println!("Loading file {:?}", path);
                  file = Some(path.to_str().unwrap().to_string());
                }
              }
            },
            _ => {},
          }
        }
      }

      // Render the `Ui` to a list of primitives that we can send to the main thread for
      // display. Wakeup `winit` for rendering.
      if let Some(primitives) = ui.draw_if_changed() {
        if render_tx.send(primitives.owned()).is_err()
        || evproxy.wakeup().is_err() {
          break 'conrod;
        }
      }
    }
  }

  // Draws the given `primitives` to the given `Display`.
  fn draw(display: &glium::Display,
          renderer: &mut conrod::backend::glium::Renderer,
          image_map: &conrod::image::Map<SrgbTexture2d>,
          primitives: &conrod::render::OwnedPrimitives) {
    renderer.fill(display, primitives.walk(), &image_map);
    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 1.0);
    renderer.draw(display, &mut target, &image_map).unwrap();
    target.finish().unwrap();
  }

  let icache = cache::ImageCache::new();
  let imap = Mutex::new(ImageMapping {
    id: None,
    img: None,
    alldone: true,
  });

  crossbeam::scope(|scope| {
    let evproxy = evloop.create_proxy();
    scope.spawn(||run_conrod(logoid, &imap, event_rx, render_tx, evproxy));

    // Run the `winit` loop.
    let mut last_update = std::time::Instant::now();
    let mut closed = false;
    let mut oldrawid: Option<conrod::image::Id> = None;
    while !closed {
      // We don't want to loop any faster than 60 FPS, so wait until it has been at least
      // 16ms since the last yield.
      let sixteen_ms = std::time::Duration::from_millis(16);
      let now = std::time::Instant::now();
      let duration_since_last_update = now.duration_since(last_update);
      if duration_since_last_update < sixteen_ms {
        std::thread::sleep(sixteen_ms - duration_since_last_update);
      }

      evloop.run_forever(|event| {
        // Use the `winit` backend feature to convert the winit event to a conrod one.
        if let Some(event) = conrod::backend::winit::convert_event(event.clone(), &display) {
          event_tx.send(event).unwrap();
        }

        match event {
          Event::WindowEvent { event, .. } => match event {
            // Break from the loop upon `Escape`.
            WindowEvent::Closed |
            WindowEvent::KeyboardInput {
              input: glium::glutin::KeyboardInput {
                virtual_keycode: Some(VirtualKeyCode::Escape),
                ..
              },
              ..
            } => {
              closed = true;
              return glium::glutin::ControlFlow::Break;
            },
            // We must re-draw on `Resized`, as the event loops become blocked during
            // resize on macOS.
            WindowEvent::Resized(..) => {
              if let Some(primitives) = render_rx.iter().next() {
                draw(&display, &mut renderer, &image_map, &primitives);
              }
            },
            _ => {},
          },
          glium::glutin::Event::Awakened => return glium::glutin::ControlFlow::Break,
          _ => (),
        }

        glium::glutin::ControlFlow::Continue
      });

      // Load images if needed
      {
        let image = {
          let imap = imap.lock().unwrap();
          if imap.alldone {
            Arc::new(None)
          } else {
            if let Some(ref id) = imap.id {
              let evproxy = evloop.create_proxy();
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
            &display,
            raw_image,
            glium::texture::SrgbFormat::U8U8U8,
            glium::texture::MipmapsOption::NoMipmap
          ).unwrap();
          if let Some(id) = oldrawid {
            image_map.remove(id);
          }
          let rawid = image_map.insert(img);
          let mut imap = imap.lock().unwrap();
          imap.img = Some((rawid, dims.0, dims.1));
          imap.alldone = true;
          oldrawid = Some(rawid);
        }
      }

      // Draw the most recently received `conrod::render::Primitives` sent from the `Ui`.
      if let Some(primitives) = render_rx.try_iter().last() {
          draw(&display, &mut renderer, &image_map, &primitives);
      }

      last_update = std::time::Instant::now();
    }

    // Make sure the conrod thread terminates so the app exits
    drop(event_tx);
  });
}

// Load the image from a file
fn load_image(buf: &[u8], display: &glium::Display) -> glium::texture::SrgbTexture2d {
  let img = image::load_from_memory(buf).unwrap().to_rgba();
  let dims = img.dimensions();
  let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(&img.into_raw(), dims);
  glium::texture::SrgbTexture2d::new(display, raw_image).unwrap()
}

fn load_font(buf: &[u8]) -> rusttype::Font {
  rusttype::FontCollection::from_bytes(buf).into_font().unwrap()
}
