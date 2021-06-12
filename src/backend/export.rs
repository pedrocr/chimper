use std::fs::File;
use std::io::BufWriter;
use image::ColorType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestedExport {
  pub file: String,
  pub ops: Option<imagepipe::PipelineOps>,
}

pub fn export_file(req: &RequestedExport) {
  let outfile = format!("{}.jpg", req.file);
  log::info!("Exporting {} into {}", req.file, outfile);

  let mut pipeline = match imagepipe::Pipeline::new_from_file(&req.file) {
    Ok(pipe) => pipe,
    Err(_) => {
      log::error!("Don't know how to load \"{}\"", req.file);
      return
    },
  };
  if let Some(ref ops) = req.ops {
    pipeline.ops = ops.clone();
  }
  let decoded = match pipeline.output_8bit(None) {
    Ok(img) => img,
    Err(_) => {
      log::error!("Processing for \"{}\" failed", req.file);
      return
    },
  };
  let uf = match File::create(&outfile) {
    Ok(val) => val,
    Err(e) => {
      log::error!("Error creating output file: {}", e);
      return;
    }
  };
  let mut f = BufWriter::new(uf);
  let mut jpg_encoder = image::jpeg::JpegEncoder::new_with_quality(&mut f, 90);
  match jpg_encoder.encode(&decoded.data, decoded.width as u32, decoded.height as u32, ColorType::Rgb8) {
    Ok(_) => {},
    Err(_) => {
      log::error!("Error encoding jpg for \"{}\"", req.file);
      return
    },
  }
  log::info!("Finished exporting {}", outfile);
}
