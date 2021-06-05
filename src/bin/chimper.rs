extern crate chimper;
use std::env;
use std::path::PathBuf;

fn usage() {
  eprintln!("Usage: chimper [file or dir]");
}

fn main() {
  env_logger::init();

  let args: Vec<_> = env::args().collect();
  if args.len() > 2 {
    log::error!("called with wrong arguments");
    usage();
    std::process::exit(1);
  }
  let path = if args.len() > 1 {
    let path = PathBuf::from(args[1].clone());
    if !path.exists() {
      log::error!("path does not exist: \"{}\"", args[1]);
      usage();
      std::process::exit(2);
    }
    if !(path.is_dir() || path.is_file()) {
      log::error!("path is not a file or a dir: \"{}\"", args[1]);
      usage();
      std::process::exit(3);
    }
    Some(path)
  } else {
    None
  };

  chimper::frontend::main::run_app(path);
}
