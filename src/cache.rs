extern crate crossbeam_utils;
use self::crossbeam_utils::thread::Scope;
extern crate imagepipe;
extern crate multicache;
use self::multicache::MultiCache;
use std::sync::Arc;
use self::imagepipe::SRGBImage;
use conrod::backend::glium::glium;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestedImage {
  pub file: String,
  pub size: usize,
  pub ops: Option<imagepipe::PipelineOps>,
}

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
  images: MultiCache<RequestedImage, Option<(SRGBImage, imagepipe::PipelineOps)>>,
}

impl ImageCache {
  pub fn new() -> ImageCache {
    ImageCache {
      images: MultiCache::new(100000000),
    }
  }

  pub fn get<'a>(&'a self, req: RequestedImage, scope: &Scope<'a>, evproxy: glium::glutin::EventsLoopProxy) -> Arc<Option<(SRGBImage, imagepipe::PipelineOps)>> {
    if let Some(img) = self.images.get(&req) {
      // We found at least an empty guard value, return that cloned to activate Arc
      img.clone()
    } else {
      // Write a None to avoid any reissues of the same thread
      self.images.put(req.clone(), None, 0);
      self.load_raw(req, scope, evproxy);
      Arc::new(None)
    }
  }

  fn load_raw<'a>(&'a self, req: RequestedImage, scope: &Scope<'a>, evproxy: glium::glutin::EventsLoopProxy) {
    let maxwidth = SIZES[req.size][0];
    let maxheight = SIZES[req.size][1];

    scope.spawn(move || {
      eprintln!("processing {}", req.file);

      let mut pipeline = match imagepipe::Pipeline::new_from_file(&req.file, maxwidth, maxheight, false) {
        Ok(pipe) => pipe,
        Err(_) => {
          eprintln!("Don't know how to load \"{}\"", req.file);
          return
        },
      };
      if let Some(ref ops) = req.ops {
        pipeline.ops = ops.clone();
      }
      let decoded = match pipeline.output_8bit() {
        Ok(img) => img,
        Err(_) => {
          eprintln!("Processing for \"{}\" failed", req.file);
          return
        },
      };
      let imgsize = decoded.width*decoded.height*3;
      let ops = pipeline.ops.clone();
      self.images.put(req.clone(), Some((decoded, ops)), imgsize);
      evproxy.wakeup().is_ok();
    });
  }
}
