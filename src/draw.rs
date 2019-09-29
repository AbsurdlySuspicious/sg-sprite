use super::*;
use crate::raise;
use image::{self, imageops, GenericImage, GenericImageView, ImageBuffer, Pixels, Rgba, RgbaImage};
use parse::*;
use std::format as fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub const SRC_EXT: &str = ".png";
pub const BLOCK_W: u32 = 32;
pub const BLOCK_H: u32 = 32;
pub const CANVAS_PAD_W: u32 = 0;
pub const CANVAS_PAD_H: u32 = 0;

pub fn draw_all(src_img: &Path, out_dir: &Path, lay: &ParsedLay) -> Result<(), PErr> {
  for (pass, idx) in lay
    .sprites
    .iter()
    .enumerate()
    .filter_map(|(n, s)| match s.t {
      SpriteT::Dep(_) => Some(n),
      _ => None,
    })
    .enumerate()
  {
    draw_dep(src_img, out_dir, lay, idx, pass)?;
  }
  Ok(())
}

pub fn draw_dep(
  src_img: &Path,
  out_dir: &Path,
  lay: &ParsedLay,
  sprite_idx: usize,
  pass: usize,
) -> Result<(), PErr> {
  if !out_dir.is_dir() {
    raise("out isn't a directory")?
  }

  let sprite_name = match src_img.file_name() {
    Some(f) => f.to_string_lossy(),
    None => return raise("wrong src file"),
  };
  let sprite_name = sprite_name.trim_end_matches(SRC_EXT);

  let name_suf = match lay.sprites.get(sprite_idx) {
    None => return raise("wrong sprite index"),
    Some(s) => match s.t {
      SpriteT::Base => fmt!("b{}", s.id),
      SpriteT::Sub => fmt!("s{}", s.id),
      SpriteT::Dep(d) => fmt!("d{}_{}", d, s.id),
    },
  };

  let out = out_dir.with_file_name(fmt!("{}_{}{}", sprite_name, name_suf, SRC_EXT));

  let mut lst: Vec<&Sprite> = Vec::with_capacity(10);

  let mut next = Some(sprite_idx);
  while let Some(n) = next {
    let s = &lay.sprites[n];
    lst.push(s);
    next = match &s.t {
      SpriteT::Base => None,
      SpriteT::Sub => Some(0), // base
      SpriteT::Dep(d) => Some(lay.sub_map[d]),
    }
  }

  draw_sprites(src_img, &out, lst.as_ref(), pass, lay)
}

pub fn draw_sprites(
  png_in: &Path,
  png_out: &Path,
  sprites_rev: &[&Sprite],
  pass: usize,
  lay: &ParsedLay,
) -> Result<(), PErr> {
  eprint!("draw {}:", pass);

  let (x_mid, y_mid) = lay.sprite_xy_min;
  let x_mid = x_mid.abs();
  let y_mid = y_mid.abs();

  eprint!("\rdraw {}: decode", pass);
  let mut src = image::open(png_in)?;

  eprint!("\rdraw {}: canvas", pass);
  let mut canvas = ImageBuffer::from_pixel(
    lay.sprite_w + CANVAS_PAD_W,
    lay.sprite_h + CANVAS_PAD_H,
    Rgba([0, 0, 0, 0]),
  );

  let mut chunk_c = 0usize;

  for sp in sprites_rev.iter().rev() {
    let chunk_s = sp.c_offset;
    let chunk_e = chunk_s + sp.c_count;
    let chunks = &lay.chunks[chunk_s..chunk_e];

    for c in chunks.iter() {
      eprint!("\rdraw {}: chunk {}", pass, chunk_c);

      let (cx, cy) = ((x_mid + c.img_x) as u32, (y_mid + c.img_y) as u32);
      let (sx, sy) = (c.chunk_x as u32 - 1, c.chunk_y as u32 - 1);
      let mut fdst = imageops::crop(&mut canvas, cx, cy, BLOCK_W, BLOCK_H);
      let fsrc = imageops::crop(&mut src, sx, sy, BLOCK_W, BLOCK_H);

      imageops::replace(&mut fdst, &fsrc, 0, 0);
      chunk_c += 1;
    }
  }

  eprint!(" [encode]");
  canvas.save(png_out)?;

  eprintln!(" [done]");
  Ok(())
}
