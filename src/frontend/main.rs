extern crate imagepipe;
extern crate conrod_core;
use conrod_core::text::Font;
extern crate glium;
//use self::glium::glutin::event_loop::EventLoopProxy;
use self::glium::texture::SrgbTexture2d;
use self::glium::Surface;
use self::glium::glutin::window::Fullscreen;

extern crate conrod_glium;
use conrod_glium::Renderer;

use std::env;
use std::path::PathBuf;
extern crate crossbeam_utils;
extern crate image;

use crate::frontend::*;
//use crate::backend::cache;
//use crate::backend::cache::RequestedImage;

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

pub struct Chimper {
  pub dragwidth: f64,
  pub sidewidth: f64,
  pub imagepadding: f64,
  pub use_sidepane: bool,
  pub logoid: conrod_core::image::Id,
  pub ids: ChimperIds,
  pub sideopt: bool,
  pub directory: std::path::PathBuf,
  pub file: Option<String>,
  pub image: Option<DisplayableImage>,
  pub ops: Option<imagepipe::PipelineOps>,
  pub selected_op: SelectedOp,
  pub fullscreen: bool,
}

impl Chimper {
  fn new(logoid: conrod_core::image::Id, path: Option<PathBuf>, ui: &mut conrod_core::Ui) -> Self {

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
      ids: ChimperIds::new(ui.widget_id_generator()),
      file,
      directory,
      sideopt,
      image: None,
      ops: None,
      selected_op: SelectedOp::None,
      fullscreen: false,
    }
  }
}

#[derive(Debug, Clone)]
pub struct DisplayableImage {
  pub id: conrod_core::image::Id,
  pub width: u32,
  pub height: u32,
}

static WIN_W: f64 = 1200.0;
static WIN_H: f64 = 800.0;

enum AppEvent {
  Fullscreen(bool),
  Sidepane,
}

pub fn run_app(path: Option<PathBuf>) {
  // Build the window.
  let event_loop = glium::glutin::event_loop::EventLoop::new();
  let window = glium::glutin::window::WindowBuilder::new()
    .with_title("Chimper")
    .with_inner_size(glium::glutin::dpi::LogicalSize::new(WIN_W, WIN_H));
  let context = glium::glutin::ContextBuilder::new()
    .with_vsync(true)
    .with_multisampling(4);
  let display = glium::Display::new(window, context, &event_loop).unwrap();
  let mut renderer = Renderer::new(&display).unwrap();
  let mut image_map = conrod_core::image::Map::new();

  // Grab a chimper logo and insert it into the image map
  let img = image::load_from_memory(logo::random()).unwrap().to_rgba8();
  let dims = img.dimensions();
  let raw = glium::texture::RawImage2d::from_raw_rgba_reversed(&img.into_raw(), dims);
  let texture = glium::texture::SrgbTexture2d::new(&display, raw).unwrap();
  let logoid = image_map.insert(texture);

  // A channel to send events from the main `winit` thread to the conrod thread.
  let (event_tx, event_rx) = std::sync::mpsc::channel();
  // A channel to send app events from the main `winit` thread to the conrod thread.
  let (app_event_tx, app_event_rx) = std::sync::mpsc::channel();
  // A channel to send `render::Primitive`s from the conrod thread to the `winit thread.
  let (render_tx, render_rx) = std::sync::mpsc::channel();
  // Clone the handle to the events loop so that we can interrupt it when ready to draw.
  let events_loop_proxy = event_loop.create_proxy();

  // A function that runs the conrod loop.
  fn run_conrod(
    event_rx: std::sync::mpsc::Receiver<conrod_core::event::Input>,
    app_event_rx: std::sync::mpsc::Receiver<AppEvent>,
    render_tx: std::sync::mpsc::Sender<conrod_core::render::OwnedPrimitives>,
    events_loop_proxy: glium::glutin::event_loop::EventLoopProxy<()>,
    logoid: conrod_core::image::Id,
    path: Option<PathBuf>,
  ) {
    // Construct our `Ui`.
    let mut ui = conrod_core::UiBuilder::new([WIN_W, WIN_H]).build();
    ui.fonts.insert(Font::from_bytes(include_bytes!("../../fonts/NotoSans-Regular.ttf")).unwrap());

    let mut chimp = Chimper::new(logoid, path, &mut ui);

    // Many widgets require another frame to finish drawing after clicks or hovers, so we
    // insert an update into the conrod loop using this `bool` after each event.
    let mut needs_update = true;
    'conrod: loop {
      // Process any app events
      while let Ok(event) = app_event_rx.try_recv() {
        match event {
          AppEvent::Fullscreen(fs) => chimp.fullscreen = fs,
          AppEvent::Sidepane => chimp.use_sidepane = !chimp.use_sidepane,
        }
      }

      // Collect any pending events.
      let mut events = Vec::new();
      while let Ok(event) = event_rx.try_recv() {
        events.push(event);
      }

      // If there are no events pending, wait for them.
      if events.is_empty() && !needs_update {
        match event_rx.recv() {
          Ok(event) => events.push(event),
          Err(_) => break 'conrod,
        };
      }
      needs_update = false;

      // Input each event into the `Ui`.
      for event in events {
        ui.handle_event(event);
        needs_update = true;
      }

      // Instantiate a GUI demonstrating every widget type provided by conrod.
      gui::draw_gui(&mut chimp, &mut ui);
      //conrod_example_shared::gui(&mut ui.set_widgets(), &ids, &mut app);

      // Render the `Ui` to a list of primitives that we can send to the main thread for
      // display. Wakeup `winit` for rendering.
      if let Some(primitives) = ui.draw_if_changed() {
        needs_update = true;
        if render_tx.send(primitives.owned()).is_err()
          || events_loop_proxy.send_event(()).is_err()
        {
          break 'conrod;
        }
      }
    }
  }

  // Draws the given `primitives` to the given `Display`.
  fn draw(
    display: &glium::Display,
    renderer: &mut Renderer,
    image_map: &conrod_core::image::Map<SrgbTexture2d>,
    primitives: &conrod_core::render::OwnedPrimitives,
  ) {
    renderer.fill(display, primitives.walk(), &image_map);
    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 1.0);
    renderer.draw(display, &mut target, &image_map).unwrap();
    target.finish().unwrap();
  }

  // Spawn the conrod loop on its own thread.
  std::thread::spawn(move || run_conrod(event_rx, app_event_rx, render_tx, events_loop_proxy, logoid, path));

  // Run the `winit` loop.
  let mut is_waken = false;
  let mut latest_primitives = None;
  let mut fullscreen = false;
  support::run_loop(display, event_loop, move |request, display| {
    match request {
      support::Request::Event {
        event,
        should_update_ui,
        should_exit,
      } => {
        // Use the `winit` backend feature to convert the winit event to a conrod one.
        if let Some(event) = support::convert_event(&event, &display.gl_window().window()) {
            event_tx.send(event).unwrap();
        }

        match event {
          glium::glutin::event::Event::WindowEvent { event, .. } => match event {
            // Break from the loop upon `Escape`.
            glium::glutin::event::WindowEvent::CloseRequested
            | glium::glutin::event::WindowEvent::KeyboardInput {
              input:
                glium::glutin::event::KeyboardInput {
                  virtual_keycode:
                    Some(glium::glutin::event::VirtualKeyCode::Escape),
                  ..
                },
              ..
            } => *should_exit = true,
            // We must re-draw on `Resized`, as the event loops become blocked during
            // resize on macOS.
            glium::glutin::event::WindowEvent::Resized(..) => {
              if let Some(primitives) = render_rx.try_iter().last() {
                latest_primitives = Some(primitives);
              }
              if let Some(primitives) = &latest_primitives {
                draw(&display, &mut renderer, &image_map, primitives);
              }
            }
            // Fullscreen on F11
            glium::glutin::event::WindowEvent::KeyboardInput {
              input:
                glium::glutin::event::KeyboardInput {
                  virtual_keycode: Some(glium::glutin::event::VirtualKeyCode::F11),
                  state: glium::glutin::event::ElementState::Pressed,
                  ..
                },
              ..
            } => {
              fullscreen = !fullscreen;
              if fullscreen {
                display.gl_window().window().set_fullscreen(Some(Fullscreen::Borderless(None)));
              } else {
                display.gl_window().window().set_fullscreen(None);
              }
              app_event_tx.send(AppEvent::Fullscreen(fullscreen)).unwrap();
            },
            // Fullscreen on F11
            glium::glutin::event::WindowEvent::KeyboardInput {
              input:
                glium::glutin::event::KeyboardInput {
                  virtual_keycode: Some(glium::glutin::event::VirtualKeyCode::Tab),
                  state: glium::glutin::event::ElementState::Pressed,
                  ..
                },
              ..
            } => {
              app_event_tx.send(AppEvent::Sidepane).unwrap();
            },
            _ => {}
          },
          glium::glutin::event::Event::UserEvent(()) => {
            is_waken = true;
            // HACK: This triggers the `SetUi` request so that we can request a redraw.
            *should_update_ui = true;
          }
          _ => {}
        }
      }
      support::Request::SetUi { needs_redraw } => {
        *needs_redraw = is_waken;
        is_waken = false;
      }
      support::Request::Redraw => {
        // Draw the most recently received `conrod_core::render::Primitives` sent from the `Ui`.
        if let Some(primitives) = render_rx.try_iter().last() {
          latest_primitives = Some(primitives);
        }
        if let Some(primitives) = &latest_primitives {
          draw(&display, &mut renderer, &image_map, primitives);
        }
      }
    }
  })
}
