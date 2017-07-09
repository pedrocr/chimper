extern crate crossbeam;
extern crate image;
extern crate rawloader;
extern crate multicache;
use self::multicache::MultiCache;
use std::sync::Arc;
use std::path::Path;
use self::rawloader::SRGBImage;

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
  images: MultiCache<(String, usize), Option<SRGBImage>>,
}

impl ImageCache {
  pub fn new() -> ImageCache {
    ImageCache {
      images: MultiCache::new(100000000),
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

  pub fn get<'a>(&'a self, path: String, size: usize, scope: &crossbeam::Scope<'a>, ui: &'a UIContext) -> Arc<Option<SRGBImage>> {
    if let Some(img) = self.images.get(&(path.clone(), size)) {
      // We found at least an empty guard value, return that cloned to activate Arc
      img.clone()
    } else {
      // Write a None to avoid any reissues of the same thread
      self.images.put((path.clone(), size), None, 0);
      self.load_raw(path, size, scope, ui);
      Arc::new(None)
    }
  }

  fn load_raw<'a>(&'a self, path: String, size: usize, scope: &crossbeam::Scope<'a>, ui: &'a UIContext) {
    let maxwidth = SIZES[size][0];
    let maxheight = SIZES[size][1];

    scope.spawn(move || {
      let decoded = match rawloader::decode(&path) {
        Ok(img) => img.to_srgb(maxwidth, maxheight).unwrap(),
        // If we couldn't load it as a raw try with the normal image loading
        Err(_) => match image::open(&Path::new(&path)) {
          Ok(img) => {
            let rgb = img.to_rgb();
            let width = rgb.width() as usize;
            let height = rgb.height() as usize;
            SRGBImage {
              data: rgb.into_raw(),
              width: width,
              height: height,
            }
          }
          Err(_) => {
            println!("Don't know how to load \"{}\"", path);
            return
          }
        },
      };
      let imgsize = decoded.width*decoded.height*3;
      self.images.put((path.clone(), size), Some(decoded), imgsize);
      ui.needs_update();
    });
  }

  pub fn get_image_dimensions<'a>(&'a self, path: String, size: usize) -> Option<(usize,usize)>{
    let maxwidth = SIZES[size][0];
    let maxheight = SIZES[size][1];

      let decoded = match rawloader::decode(&path) {
        Ok(img) => img.to_srgb(maxwidth, maxheight).unwrap(),
        // If we couldn't load it as a raw try with the normal image loading
        Err(_) => match image::open(&Path::new(&path)) {
          Ok(img) => {
            let rgb = img.to_rgb();
            let width = rgb.width() as usize;
            let height = rgb.height() as usize;
            SRGBImage {
              data: rgb.into_raw(),
              width: width,
              height: height,
            }
          }
          Err(_) => {
            println!("Don't know how to load \"{}\"", path);
            return None
          }
        },
      };
      let imgwidth = decoded.width;
      let imgheight = decoded.height;
      return Some((imgwidth, imgheight));
    }
  }
