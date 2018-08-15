extern crate crossbeam;
extern crate imagepipe;
extern crate multicache;
use self::multicache::MultiCache;
use std::sync::Arc;
use self::imagepipe::SRGBImage;
use conrod::backend::glium::glium;

const SIZES: [[usize;2];7] = [
  [640, 480],   //  0,3MP - Small thumbnail
  [1400, 800],  //  1,1MP - 720p+
  [2000, 1200], //  2,4MP - 1080p+
  [2600, 1600], //  4,2MP - WQXGA
  [4100, 2200], //  9,0MP - 4K
  [5200, 2900], // 15,1MP - 5K
  [0, 0],       // Go full size above 5K
];

pub fn smallest_size(width: usize, height: usize) -> usize {
  for (i,vals) in SIZES.iter().enumerate() {
    if vals[0] >= width && vals[1] >= height {
      return i
    }
  }
  return SIZES.len() - 1
}

pub struct ImageCache {
  images: MultiCache<(String, usize), Option<SRGBImage>>,
}

impl ImageCache {
  pub fn new() -> ImageCache {
    ImageCache {
      images: MultiCache::new(100000000),
    }
  }

  pub fn get<'a>(&'a self, path: String, size: usize, scope: &crossbeam::Scope<'a>, evproxy: glium::glutin::EventsLoopProxy) -> Arc<Option<SRGBImage>> {
    if let Some(img) = self.images.get(&(path.clone(), size)) {
      // We found at least an empty guard value, return that cloned to activate Arc
      img.clone()
    } else {
      // Write a None to avoid any reissues of the same thread
      self.images.put((path.clone(), size), None, 0);
      self.load_raw(path, size, scope, evproxy);
      Arc::new(None)
    }
  }

  fn load_raw<'a>(&'a self, path: String, size: usize, scope: &crossbeam::Scope<'a>, evproxy: glium::glutin::EventsLoopProxy) {
    let maxwidth = SIZES[size][0];
    let maxheight = SIZES[size][1];

    scope.spawn(move || {
      let decoded = match imagepipe::simple_decode_8bit(&path, maxwidth, maxheight) {
        Ok(img) => img,
        Err(_) => {
          eprintln!("Don't know how to load \"{}\"", path);
          return
        },
      };
      let imgsize = decoded.width*decoded.height*3;
      self.images.put((path.clone(), size), Some(decoded), imgsize);
      evproxy.wakeup().is_ok();
    });
  }
}
