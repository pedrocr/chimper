extern crate imagepipe;
extern crate multicache;
use self::multicache::MultiCache;
use std::sync::Arc;
use self::imagepipe::SRGBImage;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestedImage {
  pub file: String,
  pub width: u32,
  pub height: u32,
  pub ops: Option<imagepipe::PipelineOps>,
}

#[derive(Debug, Clone)]
pub struct ImageOutput {
  pub image: SRGBImage,
  pub ops: imagepipe::PipelineOps,
  pub default_ops: imagepipe::PipelineOps,
  pub maxwidth: u32,
  pub maxheight: u32,
}

#[derive(Debug, Clone)]
pub struct ImageResult {
  pub file: String,
  pub image: Option<Arc<ImageOutput>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
  pub file: String,
  pub level: usize,
  pub ops: Option<imagepipe::PipelineOps>,
}

const SIZES: [(u32, u32);7] = [
  (640, 480),   //  0,3MP - Small thumbnail
  (1400, 800),  //  1,1MP - 720p+
  (2000, 1200), //  2,4MP - 1080p+
  (2600, 1600), //  4,2MP - WQXGA
  (4100, 2200), //  9,0MP - 4K
  (5200, 2900), // 15,1MP - 5K
  (0, 0),       // Go full size above 5K
];

fn find_level(width: u32, height: u32) -> usize {
  for (i,vals) in SIZES.iter().enumerate() {
    if vals.0 >= width && vals.1 >= height {
      return i
    }
  }
  SIZES.len() - 1
}

impl CacheKey {
  fn from_request(req: RequestedImage) -> Self {
    let level = find_level(req.width, req.height);
    CacheKey {
      level,
      file: req.file,
      ops: req.ops,
    }
  }
}

pub struct ImageCache {
  images: MultiCache<CacheKey, ImageOutput>,
  opbuffers: imagepipe::PipelineCache,
}

impl ImageCache {
  pub fn new() -> ImageCache {
    ImageCache { // For now default to 100MiB for both caches
      images: MultiCache::new(100000000),
      opbuffers: imagepipe::Pipeline::new_cache(100000000),
    }
  }

  pub fn get(&self, req: RequestedImage) -> ImageResult {
    let file = req.file.clone();
    let key = CacheKey::from_request(req);
    if !self.images.contains_key(&key) {
      self.load_raw(&key);
    }
    ImageResult {
      file,
      image: self.images.get(&key),
    }
  }

  fn load_raw(&self, req: &CacheKey) {
    let (maxwidth, maxheight) = SIZES[req.level];

    log::info!("processing {}", req.file);

    let mut pipeline = match imagepipe::Pipeline::new_from_file(&req.file) {
      Ok(pipe) => pipe,
      Err(_) => {
        log::error!("Don't know how to load \"{}\"", req.file);
        return
      },
    };
    pipeline.globals.settings.maxwidth = maxwidth as usize;
    pipeline.globals.settings.maxheight = maxheight as usize;
    let default_ops = pipeline.ops.clone();
    if let Some(ref ops) = req.ops {
      pipeline.ops = ops.clone();
    }
    let decoded = match pipeline.output_8bit(Some(&self.opbuffers)) {
      Ok(img) => img,
      Err(_) => {
        log::error!("Processing for \"{}\" failed", req.file);
        return
      },
    };
    let imgsize = decoded.width*decoded.height*3;
    let maxsize = if decoded.width < maxwidth as usize && decoded.height < maxheight as usize {
      // This is already native size, there's no point in asking us for larger
      (u32::MAX, u32::MAX)
    } else {
      (maxwidth, maxheight)
    };
    let value = Arc::new(ImageOutput {
      image: decoded,
      ops: pipeline.ops.clone(),
      default_ops,
      maxwidth: maxsize.0,
      maxheight: maxsize.1,
    });
    if req.ops.is_none() {
      // We have requested an image with default ops so also store in the cache
      // with the ops themselves. Otherwise we would waste time running the whole
      // pipeline just to find an image that we already have.
      let mut newreq = req.clone();
      newreq.ops = Some(pipeline.ops.clone());
      // This reduces available cache space when in reality the storage is shared
      // thanks to Arc. The old Multicache aliasing stuff would fix that but it
      // seems like too much complexity for a small gain.
      self.images.put_arc(newreq, value.clone(), imgsize);
    }
    self.images.put_arc(req.clone(), value.clone(), imgsize);
  }
}
