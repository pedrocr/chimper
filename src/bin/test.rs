extern crate chimper;

#[macro_use] extern crate conrod;
use conrod::backend::glium::glium;
use conrod::{Positionable, Widget};

// We use the normal macro to create our GUI id's
widget_ids!(
struct AppIds {
  circle
});

// Our app GUI state is just the normal struct
struct App {
  show: bool,
  ids: Option<AppIds>,
}

// We can initialize our GUI state however we want
impl App {
  fn new() -> Self {
    Self {
      show: true,
      ids: None,
    }
  }
}

// This is where we implement our app's GUI behaviors by implementing the ChimperApp trait
impl chimper::window::ChimperApp for App {
  fn initialize(&mut self, ui: &mut conrod::Ui) {
    // Here we initialize anything in our app that needs the Ui

    // FIXME: We need to initialize the id's by hand because it's our own struct that uses the Ui
    self.ids = Some(AppIds::new(ui.widget_id_generator()));
  }

  fn draw_gui(&mut self, ui: &mut conrod::Ui, evproxy: &glium::glutin::EventsLoopProxy) {
    // Here we draw our GUI itself

    // FIXME: this again an ugly bit that should disappear
    let ids = match self.ids {
      Some(ref ids) => ids,
      None => unreachable!(),
    };

    let ui = &mut ui.set_widgets();
    if self.show {
      conrod::widget::Circle::fill(40.0).middle().set(ids.circle, ui);
    }

    // We can use the proxy to wake up the event loop and force a redraw
    evproxy.wakeup().is_ok();
  }

  fn process_event(&mut self, event: &conrod::event::Input) {
    // Here we can use the events to change stuff in our app manually
    match *event {
      conrod::event::Input::Press(conrod::input::Button::Keyboard(conrod::input::Key::Tab)) => {
        self.show = !self.show;
      },
      _ => (),
    }
  }
}

// The main can just be a thin launcher
fn main() {
  let mut window = chimper::window::ChimperWindow::new("Testing", 600, 600);
  let mut app = App::new();

  window.run(&mut app, move |_display, _renderer, _image_map, _evproxy| {
    // Do stuff in the winit loop after event processing and before drawing like for
    // example adding/swapping image textures to the image_map
  });
}
