#![allow(dead_code, unused_imports)]

use custom_error::custom_error;
use std::env;
use std::format as fmt;
use std::fs::File;
use std::io;

mod draw;
mod parse;

use draw::*;
use parse::*;
use std::path::{Path, PathBuf};

custom_error! { pub PErr
  IO{source: io::Error} = "IO: {source}",
  Img{source: image::ImageError} = "image: {source}",
  Etc{msg: String} = "{msg}",
}

fn raise<T>(m: impl Into<String>) -> Result<T, PErr> {
  Err(PErr::Etc { msg: m.into() })
}

const LAY_EXT: &str = "_.lay";

fn main_() -> Result<(), PErr> {
  let args: Vec<_> = env::args().collect();

  if args.len() < 2 {
    println!("Usage: <OUT_DIR> <LAY_FILE> [LAY_FILES...]");
    return raise("wrong args");
  }

  let png_o = Path::new(&args[0]);

  for lay_i in &args[1..] {
    let png_p = fmt!("{}{}", lay_i.trim_end_matches(LAY_EXT), SRC_EXT);
    let png_i = Path::new(&png_p);

    println!("open: {}", png_i.file_name().and_then(|f| f.to_str()).unwrap_or(""));

    let mut lay_f = File::open(lay_i)?;
    let lay = parse_lay(&mut lay_f)?;

    draw_all(&png_i, &png_o, &lay)?;
  }

  Ok(())
}

fn main() {
  if let Err(e) = main_() {
    eprintln!("[E] {}", e);
  }
}
