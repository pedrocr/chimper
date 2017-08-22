extern crate conrod;
use conrod::backend::glium::glium;
use conrod::backend::glium::glium::Surface;

fn main() {
  // The initial width and height in "points".
  const WIN_W: u32 = 400;
  const WIN_H: u32 = 400;

  // Build the window.
  let mut events_loop = glium::glutin::EventsLoop::new();
  let window = glium::glutin::WindowBuilder::new()
    .with_title("Conrod with glium!")
    .with_dimensions(WIN_W, WIN_H);
  let context = glium::glutin::ContextBuilder::new()
    .with_vsync(true)
    .with_multisampling(4);
  let display = glium::Display::new(window, context, &events_loop).unwrap();

  // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
  // for drawing to the glium `Surface`.
  //
  // Internally, the `Renderer` maintains:
  // - a `backend::glium::GlyphCache` for caching text onto a `glium::texture::Texture2d`.
  // - a `glium::Program` to use as the shader program when drawing to the `glium::Surface`.
  // - a `Vec` for collecting `backend::glium::Vertex`s generated when translating the
  // `conrod::render::Primitive`s.
  // - a `Vec` of commands that describe how to draw the vertices.
  let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

  let image_map = conrod::image::Map::new();

  // A channel to send events from the main `winit` thread to the conrod thread.
  let (event_tx, event_rx) = std::sync::mpsc::channel();
  // A channel to send `render::Primitive`s from the conrod thread to the `winit thread.
  let (render_tx, render_rx) = std::sync::mpsc::channel();
  // Clone the handle to the events loop so that we can interrupt it when ready to draw.
  let events_loop_proxy = events_loop.create_proxy();

  // A function that runs the conrod loop.
  fn run_conrod(event_rx: std::sync::mpsc::Receiver<conrod::event::Input>,
                render_tx: std::sync::mpsc::Sender<conrod::render::OwnedPrimitives>,
                events_loop_proxy: glium::glutin::EventsLoopProxy) {
    // Construct our `Ui`.
    let mut ui = conrod::UiBuilder::new([WIN_W as f64, WIN_H as f64]).build();

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

      // Render the `Ui` to a list of primitives that we can send to the main thread for
      // display. Wakeup `winit` for rendering.
      if let Some(primitives) = ui.draw_if_changed() {
          if render_tx.send(primitives.owned()).is_err()
          || events_loop_proxy.wakeup().is_err() {
              break 'conrod;
          }
      }
    }
  }

  // Draws the given `primitives` to the given `Display`.
  fn draw(display: &glium::Display,
          renderer: &mut conrod::backend::glium::Renderer,
          image_map: &conrod::image::Map<glium::Texture2d>,
          primitives: &conrod::render::OwnedPrimitives) {
    renderer.fill(display, primitives.walk(), &image_map);
    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 1.0);
    renderer.draw(display, &mut target, &image_map).unwrap();
    target.finish().unwrap();
  }

  // Spawn the conrod loop on its own thread.
  std::thread::spawn(move || run_conrod(event_rx, render_tx, events_loop_proxy));
 
  // Run the `winit` loop.
  let mut awake_count = 0;
  let mut draw_count = 0;
  loop {
    let (wakeup_tx, wakeup_rx) = std::sync::mpsc::channel();

    let evproxy = events_loop.create_proxy();
    std::thread::spawn(move || {
      // BUG: This should ensure we are busy looping forever but doesn't
      evproxy.wakeup().ok();
      wakeup_tx.send(true).unwrap();
    });

    // Make sure wakeup has been called
    wakeup_rx.recv().unwrap();

    events_loop.run_forever(|event| {
      // Use the `winit` backend feature to convert the winit event to a conrod one.
      if let Some(event) = conrod::backend::winit::convert_event(event.clone(), &display) {
          event_tx.send(event).unwrap();
      }

      match event {
        glium::glutin::Event::Awakened => {
          awake_count += 1;
          eprintln!("Awakening num {}", awake_count);
          return glium::glutin::ControlFlow::Break
        },
        _ => (),
      }

      glium::glutin::ControlFlow::Continue
    });

    draw_count += 1;
    eprintln!("Draw num {}", draw_count);
    // Draw the most recently received `conrod::render::Primitives` sent from the `Ui`.
    if let Some(primitives) = render_rx.try_iter().last() {
        draw(&display, &mut renderer, &image_map, &primitives);
    }
  }
}
