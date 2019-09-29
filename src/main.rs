#![allow(dead_code, unused_imports)]

use custom_error::custom_error;
use std::env;
use std::fmt::Display;
use std::format as fmt;
use std::fs::File;
use std::io;

mod dep;
mod draw;
mod parse;

use dep::*;
use draw::*;
use parse::*;
use std::path::{Path, PathBuf};

custom_error! { pub PErr
  IO{source: io::Error} = "IO: {source}",
  Img{source: image::ImageError} = "image: {source}",
  Etc{msg: String} = "{msg}",
}

#[inline]
fn raise_e(m: impl Into<String>) -> PErr {
  PErr::Etc { msg: m.into() }
}

#[inline]
fn raise<T>(m: impl Into<String>) -> Result<T, PErr> {
  Err(raise_e(m))
}

fn print_err(e: impl Display) {
  eprintln!("[E] {}", e);
}

const LAY_EXT: &[&str] = &["_.lay", ".lay"];
const SRC_EXT: &str = ".png";

fn main() {
  if let Err(e) = main_() {
    print_err(e);
  }
}

fn main_() -> Result<(), PErr> {
  let args: Vec<_> = env::args().collect();

  if args.len() < 2 {
    println!("Usage: <OUT_DIR> <LAY_FILE> [LAY_FILES...]");
    return raise("wrong args");
  }

  let out_dir = Path::new(&args[1]);

  if !out_dir.is_dir() {
    raise("out_dir isn't a directory")?
  }

  let total = args.len() - 2;
  let mut lay_counter = 0usize;

  let status = |c: usize| move |t: &str| println!("[{}/{}] {}", c, total, t);

  for lay_i in &args[2..] {
    if let Err(e) = lay_in(&out_dir, lay_i, status(lay_counter)) {
      print_err(e);
    }

    lay_counter += 1;
  }

  Ok(())
}

fn lay_in(out_dir: &Path, lay_path: &String, status: impl Fn(&str)) -> Result<(), PErr> {
  let lay_i = Path::new(lay_path);

  let (lay_fn, lay_ext) = lay_i
    .file_name()
    .and_then(|n| n.to_str())
    .and_then(|f| LAY_EXT.iter().find(|e| f.ends_with(**e)).map(|e| (f, e)))
    .ok_or(raise_e("not a lay file"))?;

  let sprite_name = lay_fn.trim_end_matches(lay_ext);
  let src_fn = fmt!("{}{}", sprite_name, SRC_EXT);
  let src_pa: PathBuf = {
    let mut pb = PathBuf::new();
    let p = lay_i.parent().ok_or(raise_e("no parent"))?;
    pb.push(p);
    pb.push(src_fn);
    pb
  };

  status(sprite_name);

  let lay = parse_lay(&mut File::open(lay_i)?)?;
  let dep_refs = resolve_rc(&lay);
  let leafs = leaf_sprites(&dep_refs);

  /*eprint!("draw: decode");
  let mut src = draw_prep(&src_pa)?;*/

  for (pass, sp) in leafs.enumerate() {
    let s = sp.s;
    let name_suf = match s.t {
      SpriteT::Base => fmt!("b{}", s.id),
      SpriteT::Sub => fmt!("s{}", s.id),
      SpriteT::Dep(d) => fmt!("d{}_{}", d, s.id),
      SpriteT::Overlay => fmt!("o{}", s.id),
    };

    let mut out = PathBuf::new();
    out.push(&out_dir);
    out.push(fmt!("{}_{}{}", sprite_name, name_suf, SRC_EXT));

    let dep_lst = resolve_dep_list(&dep_refs, sp)?;
    /*if let Err(e) = draw_sprites(&mut src, &out, dep_lst.as_ref(), pass, &lay) {
      print_err(e);
    }*/
  }

  Ok(())
}
