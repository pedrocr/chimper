extern crate crossbeam;
extern crate image;
use self::image::{ImageBuffer, Rgba};
extern crate rawloader;
use std::sync::RwLock;
use std::sync::Arc;
use std::collections::HashMap;

use event::UIContext;

const SIZES: [[usize;2];7] = [
  [640, 480],   //  0,3MP - Small thumbnail
  [1400, 800],  //  1,1MP - 720p+
  [2000, 1200], //  2,4MP - 1080p+
  [2600, 1600], //  4,2MP - WQXGA
  [4100, 2200], //  9,0MP - 4K
  [5200, 2900], // 15,1MP - 5K
  [0, 0],       // Go full size above 5K
];

pub struct ImageCache {
  images: RwLock<HashMap<(String, usize), Option<Arc<ImageBuffer<Rgba<u8>, Vec<u8>>>>>>,
}

impl ImageCache {
  pub fn new() -> ImageCache {
    ImageCache {
      images: RwLock::new(HashMap::new()),
    }
  }

  pub fn smallest_size(&self, width: usize, height: usize) -> usize {
    for (i,vals) in SIZES.iter().enumerate() {
      if vals[0] >= width && vals[1] >= height {
        return i
      }
    }
    return SIZES.len() - 1
  }

  pub fn get<'a>(&'a self, path: &'a str, size: usize, scope: &crossbeam::Scope<'a>, ui: &'a UIContext) -> Option<Arc<ImageBuffer<Rgba<u8>, Vec<u8>>>> {
    if let Some(img) = self.images.read().unwrap().get(&(path.to_string(), size)) {
      // We found at least an empty guard value, return that cloned to activate Arc
      return img.clone()
    }

    // Write a None to avoid any reissues of the same thread
    self.images.write().unwrap().insert((path.to_string(), size), None);
    self.load_raw(path, size, scope, ui);
    None
  }

  fn load_raw<'a>(&'a self, path: &'a str, size: usize, scope: &crossbeam::Scope<'a>, ui: &'a UIContext) {
    let file = path.to_string();
    let maxwidth = SIZES[size][0];
    let maxheight = SIZES[size][1];
    let images = &self.images;

    scope.spawn(move || {
      let decoded = rawloader::decode(path).unwrap().to_linear_rgb(maxwidth, maxheight).unwrap();
      // Convert f32 RGB into u8 RGBA
      let mut buffer = vec![0 as u8; (decoded.width*decoded.height*4) as usize];
      for (pixin, pixout) in decoded.data.chunks(3).zip(buffer.chunks_mut(4)) {
        pixout[0] = (pixin[0]*255.0).max(0.0).min(255.0) as u8;
        pixout[1] = (pixin[1]*255.0).max(0.0).min(255.0) as u8;
        pixout[2] = (pixin[2]*255.0).max(0.0).min(255.0) as u8;
        pixout[3] = 255;
      }
      let img = ImageBuffer::from_raw(decoded.width as u32, decoded.height as u32, buffer).unwrap();
      images.write().unwrap().insert((file, size), Some(Arc::new(img)));
      ui.needs_update();
    });
  }
}
