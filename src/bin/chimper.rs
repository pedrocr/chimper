extern crate chimper;
use std::env;
use std::path::PathBuf;

fn main() {
  let args: Vec<_> = env::args().collect();
  if args.len() > 2 {
    eprintln!("ERROR: called with wrong arguments");
    eprintln!("Usage: chimper [file or dir]");
    std::process::exit(1);
  }
  let path = if args.len() > 1 {
    let path = PathBuf::from(args[1].clone());
    if !path.exists() {
      eprintln!("ERROR: path does not exist: \"{}\"", args[1]);
      std::process::exit(2);
    }
    if !(path.is_dir() || path.is_file()) {
      eprintln!("ERROR: path is not a file or a dir: \"{}\"", args[1]);
    }
    Some(path)
  } else {
    None
  };

  chimper::frontend::main::run_app(path);
}
