use std;
use conrod::backend::glium::glium;

/// This `Iterator`-like type simplifies some of the boilerplate involved in setting up a
/// glutin+glium event loop that works efficiently with conrod.
pub struct EventLoop {
  ui_needs_update: bool,
  last_update: std::time::Instant,
}

impl EventLoop {
  pub fn new() -> Self {
    EventLoop {
      last_update: std::time::Instant::now(),
      ui_needs_update: false,
    }
  }

  /// Produce an iterator yielding all available events.
  pub fn next(&mut self, display: &glium::Display) -> Vec<glium::glutin::Event> {
    // We don't want to loop any faster than 60 FPS, so wait until it has been at least 16ms
    // since the last yield.
    let last_update = self.last_update;
    let sixteen_ms = std::time::Duration::from_millis(16);
    let duration_since_last_update = std::time::Instant::now().duration_since(last_update);
    if duration_since_last_update < sixteen_ms {
      std::thread::sleep(sixteen_ms - duration_since_last_update);
    }

    // Collect all pending events.
    let mut events = Vec::new();
    events.extend(display.poll_events());

    // If there are no events and the `Ui` does not need updating, wait for the next event.
    if events.is_empty() && !self.ui_needs_update {
      events.extend(display.wait_events().next());
    }

    self.ui_needs_update = false;
    self.last_update = std::time::Instant::now();

    events
  }
}
