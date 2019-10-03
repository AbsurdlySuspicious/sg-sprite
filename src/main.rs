#![allow(dead_code, unused_imports)]

use custom_error::custom_error;
use std::env;
use std::fmt::Display;
use std::format as fmt;
use std::fs::File;
use std::io;
use structopt::StructOpt;

mod dep;
mod draw;
mod parse;

use crate::parse::SpriteT::Overlay;
use dep::*;
use draw::*;
use parse::*;
use std::path::{Path, PathBuf};

#[derive(StructOpt, Debug)]
#[structopt()]
struct Opts {
  /// Output dir
  #[structopt(short, long, parse(from_os_str))]
  dir: Option<PathBuf>,

  /// Limit variants to draw per sprite
  #[structopt(short, long)]
  limit: Option<usize>,

  /// .lay files to parse
  #[structopt(name = "LAY_FILES", parse(from_os_str))]
  lay_files: Vec<PathBuf>,

  /// Perform parsing only to test for errors.
  /// Do not compose actual images
  #[structopt(long)]
  dry_run: bool,
}

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
  let o = Opts::from_args();
  let out_dir = &o.dir;

  match out_dir {
    Some(d) if !d.is_dir() => return raise("out_dir isn't a directory"),
    None if !o.dry_run => {
      return raise("Output dir should be specified (-d)\nSee --help for details")
    }
    _ => (),
  }

  let total = o.lay_files.len();
  let mut lay_counter = 0usize;

  if total == 0 {
    return raise("no .lay files provided");
  }

  let status = |c: usize| move |t: &str| println!("[{}/{}] {}", c + 1, total, t);

  for lay_i in &o.lay_files {
    if let Err(e) = lay_in(out_dir, lay_i, o.limit, !o.dry_run, status(lay_counter)) {
      print_err(e);
    }

    lay_counter += 1;
  }

  Ok(())
}

fn lay_in(
  out_dir: &Option<PathBuf>,
  lay_i: &Path,
  limit: Option<usize>,
  draw_en: bool,
  status: impl Fn(&str),
) -> Result<(), PErr> {
  let limit_en = limit.is_some();
  let limit = limit.unwrap_or(0);

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

  let mut src = if draw_en {
    Some(draw_prep(&src_pa)?)
  } else {
    None
  };

  for (pass, sp) in leafs.enumerate() {
    if limit_en && pass >= limit {
      eprintln!("[I] Limit reached, proceeding to next sprite");
      break;
    }

    if sp.s.t == Overlay {
      eprintln!("[W] Overlays are unsupported, skipping");
      continue;
    }

    let s = sp.s;
    let name_suf = match s.t {
      SpriteT::Base => fmt!("b{}", s.id),
      SpriteT::Sub => fmt!("s{}", s.id),
      SpriteT::Dep { st, dep } => fmt!("t{}_d{}_{}", st, dep, s.id),
      SpriteT::Overlay => fmt!("o{}", s.id),
    };

    let dep_lst = resolve_dep_list(&dep_refs, sp)?;

    if let Some(src_i) = src.as_mut() {
      let mut out = PathBuf::new();
      out.push(out_dir.as_ref().unwrap());
      out.push(fmt!("{}_{}{}", sprite_name, name_suf, SRC_EXT));

      if let Err(e) = draw_sprites(src_i, &out, dep_lst.as_ref(), pass + 1, &lay) {
        print_err(e);
      }
    }
  }

  Ok(())
}
