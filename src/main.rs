#[macro_use] extern crate conrod;
use conrod::{widget, Colorable, Positionable, Sizeable, Borderable, Widget, color, Scalar};
use conrod::backend::glium::glium;
use conrod::backend::glium::glium::{DisplayBuild, Surface};
use conrod::backend::glium::glium::glutin::{Event, ElementState, VirtualKeyCode};
use std::path::Path;
use std::env;
extern crate crossbeam;
extern crate image;
extern crate rusttype;

mod cache;
mod logo;
mod event;

fn main() {
  let mut file: Option<String> = None;
  let currdir = env::current_dir().unwrap();
  let directory = currdir.as_path();

  const WIDTH: u32 = 1200;
  const HEIGHT: u32 = 800;
  //let mut fullscreen = false;

  // Build the window.
  let display = glium::glutin::WindowBuilder::new()
    .with_vsync()
    .with_dimensions(WIDTH, HEIGHT)
    //.with_maximized(true)
    //.with_fullscreen_windowed(fullscreen)
    .with_title("Chimper")
    .build_glium()
    .unwrap();

  let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

  // Add a `Font` to the `Ui`'s `font::Map` from file.
  ui.fonts.insert(load_font(include_bytes!("../fonts/NotoSans-Regular.ttf")));

  // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
  // for drawing to the glium `Surface`.
  let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

  // The `WidgetId` for our background and `Image` widgets.
  widget_ids!(struct Ids { background, imgcanvas, dragcanvas, leftarea, lefttext, footer,setcanvas, settop, setcont, raw_image, chimper, filenav });
  let ids = Ids::new(ui.widget_id_generator());

  let mut image_map = conrod::image::Map::new();
  let logoid = image_map.insert(load_image(logo::random(), &display));
  let mut rawid: Option<conrod::image::Id> = None;

  let mut currsize = 0 as usize;
  let mut changed_image = true;
  let dragwidth = 10.0;
  let sidewidth = 600.0;
  let mut use_sidepane = true;
  let imagepadding = 20.0;
  let icache = cache::ImageCache::new();
  let context = event::UIContext::new(&display);
  crossbeam::scope(|scope| {
    // Poll events from the window.
    'main: loop {
      // Handle all events.
      if let Some(event) = context.next(&display) {
        // Use the `winit` backend feature to convert the winit event to a conrod one.
        if let Some(event) = conrod::backend::winit::convert(event.clone(), &display) {
            ui.handle_event(event);
        }

        match event {
          // Break from the loop upon `Escape`.
          Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) |
          Event::Closed => {
            break 'main
          },
          Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Tab)) => {
            use_sidepane = !use_sidepane;
          },
//          Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::F11)) => {
//            fullscreen = !fullscreen;
//            display.get_window().unwrap().set_fullscreen_windowed(fullscreen)
//          },
          _ => {},
        }
      }

      let (width,height) = display.get_window().unwrap().get_inner_size_pixels().unwrap();
      let size = icache.smallest_size(width as usize, height as usize);

      if (size != currsize || changed_image) && file.is_some() {
        let image = icache.get(file.clone().unwrap(), size, scope, &context);
        match *(image) {
          None => {},
          Some(ref imgbuf) => {
            let dims = (imgbuf.width as u32, imgbuf.height as u32);
            let raw_image = glium::texture::RawImage2d::from_raw_rgb_reversed(&imgbuf.data, dims);
            let img = glium::texture::SrgbTexture2d::with_format(
              &display,
              raw_image,
              glium::texture::SrgbFormat::U8U8U8,
              glium::texture::MipmapsOption::NoMipmap
            ).unwrap();
            if let Some(id) = rawid {
              image_map.replace(id, img);
            } else {
              rawid = Some(image_map.insert(img));
            }
            changed_image = false;
            currsize = size;
            ui.needs_redraw();
          },
        }
      }

      let sidewidth = sidewidth * ((use_sidepane as u8) as f64);
      {
        let ui = &mut ui.set_widgets();

        // Construct our main `Canvas` tree.
        widget::Canvas::new().flow_right(&[
          (ids.leftarea, widget::Canvas::new().color(color::CHARCOAL).border(0.0).flow_down(&[
              (ids.imgcanvas, widget::Canvas::new().color(color::GREY).border(0.0)),
              (ids.footer, widget::Canvas::new().color(color::CHARCOAL).length(60.0).border(0.0)),
              ])),
          (ids.dragcanvas, widget::Canvas::new().length(dragwidth).color(color::BLACK).border(0.0)),
          (ids.setcanvas, widget::Canvas::new().length(sidewidth).border(0.0).flow_down(&[
            (ids.settop, widget::Canvas::new().color(color::GREY).length(100.0).border(0.0)),
            (ids.setcont, widget::Canvas::new().color(color::GREY).border(0.0)),
          ])),
        ]).border(0.0).set(ids.background, ui);

        if let Some(rawid) = rawid {
          let mut width = width as f64;
          let mut height = height as f64;
          match image_map.get(&rawid) {
            None => {},
            Some(img) => {
              let (maxw, maxh) = img.dimensions();
              let scale = (maxw as f64)/(maxh as f64);
              width = (ui.w_of(ids.imgcanvas).unwrap() - imagepadding).min(maxw as f64);
              height = (ui.h_of(ids.imgcanvas).unwrap() - imagepadding).min(maxh as f64);
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
            .top_right_with_margin_on(ids.settop, 6.0)
            .set(ids.chimper, ui);
        }

        const PAD: Scalar = 20.0;

        if let Some(ref f) = file {
            if let Some(file_name) = Path::new(f).file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    let option_dimensions: Option<(usize, usize)> = icache.get_image_dimensions(
                        f.clone(),
                        size);

                    if let Some(dimensions) = option_dimensions {
                        let width_usize = dimensions.0;
                        let height_usize = dimensions.1;

                        let width = &width_usize.to_string();
                        let height = &height_usize.to_string();
                        let output = format!(
                            "{} {}x{}", file_name_str,
                            width,
                            height);

                        widget::Text::new(&output)
                        .color(color::WHITE)
                        .font_size(18)
                        .padded_w_of(ids.footer, PAD)
                        .mid_top_with_margin_on(ids.footer, PAD)
                        .center_justify()
                        .line_spacing(10.0)
                        .set(ids.lefttext, ui);
                    }
                }
            }
        }

        for event in widget::FileNavigator::all(&directory)
          .color(conrod::color::LIGHT_BLUE)
          .font_size(16)
          .kid_area_wh_of(ids.setcont)
          .middle_of(ids.setcont)
          //.show_hidden_files(true)  // Use this to show hidden files
          .set(ids.filenav, ui)
        {
          println!("Caught event {:?}", event);
          match event {
            conrod::widget::file_navigator::Event::ChangeSelection(pbuf) => {
              if pbuf.len() > 0 {
                let path = pbuf[0].as_path();
                if path.is_file() {
                  println!("Loading file {:?}", path);
                  file = Some(path.to_str().unwrap().to_string());
                  changed_image = true;
                }
              }
            },
            _ => {},
          }
          context.needs_update();
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
fn load_image(buf: &[u8], display: &glium::Display) -> glium::texture::SrgbTexture2d {
  let img = image::load_from_memory(buf).unwrap().to_rgba();
  let dims = img.dimensions();
  let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(&img.into_raw(), dims);
  glium::texture::SrgbTexture2d::new(display, raw_image).unwrap()
}

fn load_font(buf: &[u8]) -> rusttype::Font {
  rusttype::FontCollection::from_bytes(buf).into_font().unwrap()
}
