use conrod::backend::glium::glium;
use conrod::backend::glium::glium::glutin;

pub struct UIContext {
  proxy: glutin::WindowProxy,
}

impl UIContext {
  pub fn new(display: &glium::Display) -> Self {
    UIContext {
      proxy: display.get_window().unwrap().create_window_proxy(),
    }
  }

  pub fn next(&self, display: &glium::Display) -> Option<glium::glutin::Event> {
    display.wait_events().next()
  }

  pub fn needs_update(&self) {
    self.proxy.wakeup_event_loop()
  }
}
