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
use std::path::{PathBuf, Path};

custom_error! { pub PErr
  IO{source: io::Error} = "IO: {source}",
  Img{source: image::ImageError} = "image: {source}",
  Parseint{source: std::num::ParseIntError} = "{source}",
  Etc{msg: String} = "{msg}"
}

fn raise<T>(m: impl Into<String>) -> Result<T, PErr> {
  Err(PErr::Etc { msg: m.into() })
}

fn main_() -> Result<(), PErr> {
  let a: Vec<_> = env::args().collect();
  let lay_i = a[1].as_str();
  let png_i = Path::new(&a[2]);
  let png_o = PathBuf::from("tmp_out.png");

  let sprite_n: usize = a[3].parse()?;

  let mut lay_f = File::open(lay_i)?;
  let lay = parse_lay(&mut lay_f)?;

  draw_sprite(png_i, &png_o, sprite_n, &lay)?;

  Ok(())
}

fn main() {
  if let Err(e) = main_() {
    eprintln!("[E] {}", e);
  }
}
