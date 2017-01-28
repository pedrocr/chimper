use std::sync::{Arc, Mutex, Condvar};
use std::time::Duration;
use conrod::backend::glium::glium;

pub struct UIContext {
  pair: Arc<(Mutex<bool>, Condvar)>,
}

impl UIContext {
  pub fn new() -> Self {
    UIContext {
      pair: Arc::new((Mutex::new(false), Condvar::new())),
    }
  }

  /// Produce an iterator yielding all available events.
  pub fn next(&self, display: &glium::Display) -> Vec<glium::glutin::Event> {
    let mut events = Vec::new();

    // FIXME: This will busy loop at 60FPS, ideally there would be a way to have the glium
    //        display also fire needs_update() calls and then we could bump the timeout
    //        to something much higher or even just use wait()
    loop {
      events.extend(display.poll_events());
      if !events.is_empty() {
        break;
      }

      let &(ref lock, ref cvar) = &*self.pair;
      let guard = lock.lock().unwrap();
      let mut needs_update = cvar.wait_timeout(guard, Duration::from_millis(16)).unwrap().0;

      if *needs_update {
        *needs_update = false;
        break;
      }
    }

    events
  }

  pub fn needs_update(&self) {
    let &(ref lock, ref cvar) = &*self.pair;
    let mut guard = lock.lock().unwrap();
    *guard = true;
    cvar.notify_one();
  }
}
