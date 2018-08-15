extern crate conrod;
extern crate crossbeam_utils;
use conrod::backend::glium::glium::{self, Surface};

fn main() {
  let mut evloop = glium::glutin::EventsLoop::new();
  let window = glium::glutin::WindowBuilder::new()
    .with_title("Test")
    .with_dimensions(glium::glutin::dpi::LogicalSize::new(400.0, 400.0));
  let context = glium::glutin::ContextBuilder::new()
    .with_vsync(true)
    .with_multisampling(4);
  let display = glium::Display::new(window, context, &evloop).unwrap();
  let _renderer = conrod::backend::glium::Renderer::new(&display).unwrap();
  let mut target = display.draw();
  target.clear_color(0.0, 0.0, 0.0, 1.0);

  crossbeam_utils::thread::scope(|_scope| {
    let evproxy = evloop.create_proxy();
    let mut numwakes = 0;
    loop {
      evproxy.wakeup().is_ok();
      std::thread::sleep(std::time::Duration::from_millis(1000));
      evloop.run_forever(|event| {
          match event {
              glium::glutin::Event::WindowEvent { event, .. } => match event {
                  // Break from the loop upon `Escape`.
                  glium::glutin::WindowEvent::Destroyed |
                  glium::glutin::WindowEvent::KeyboardInput {
                      input: glium::glutin::KeyboardInput {
                          virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                          ..
                      },
                      ..
                  } => {
                      return glium::glutin::ControlFlow::Break;
                  },
                  _ => {},
              },
              glium::glutin::Event::Awakened => {
                numwakes += 1;
                eprintln!("numwakes is {}", numwakes);
              },
              _ => (),
          }

          glium::glutin::ControlFlow::Continue
      });
    }
  });
}
