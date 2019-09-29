use super::*;
use crate::raise;
use image::{self, imageops, GenericImage, GenericImageView, ImageBuffer, Pixels, Rgba, RgbaImage, DynamicImage};
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

pub fn decode_img(img: &Path) -> Result<DynamicImage, PErr> {
  Ok(image::open(img)?)
}

pub fn draw_all(src_img: &Path, out_dir: &Path, lay: &ParsedLay) -> Result<(), PErr> {

  if !out_dir.is_dir() {
    raise("out isn't a directory")?
  }

  let sprite_name = match src_img.file_name() {
    Some(f) => f.to_string_lossy(),
    None => return raise("wrong src file"),
  };
  let sprite_name = sprite_name.trim_end_matches(SRC_EXT);

  eprint!("\rdraw: decode");
  let mut src = decode_img(src_img)?;

  let mut pass = 0usize;
  for sp in &lay.sprites
  {
    let name_suf = match sp.t {
      //SpriteT::Base => fmt!("b{}", sp.id),
      //SpriteT::Sub => fmt!("s{}", sp.id),
      SpriteT::Dep(d) => fmt!("d{}_{}", d, sp.id),
      _ => continue,
    };

    let mut out = PathBuf::new();
    out.push(&out_dir);
    out.push(fmt!("{}_{}{}", sprite_name, name_suf, SRC_EXT));

    let mut lst: Vec<&Sprite> = Vec::with_capacity(10);

    let mut next = Some(sp);
    while let Some(s) = next {
      lst.push(s);
      let ni = match &s.t {
        SpriteT::Base => None,
        SpriteT::Sub => Some(0), // base
        SpriteT::Dep(d) => Some(lay.sub_map[d]),
      };
      next = ni.map(|i| &lay.sprites[i]);
    }

    draw_sprites(&mut src, &out, lst.as_ref(), pass, lay)?;
    pass += 1;
  }

  Ok(())
}

pub fn draw_sprites(
  src: &mut DynamicImage,
  png_out: &Path,
  sprites_rev: &[&Sprite],
  pass: usize,
  lay: &ParsedLay,
) -> Result<(), PErr> {
  eprint!("draw {}:", pass);

  let (x_mid, y_mid) = lay.sprite_xy_min;
  let x_mid = x_mid.abs();
  let y_mid = y_mid.abs();

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
      let fsrc = imageops::crop(src, sx, sy, BLOCK_W, BLOCK_H);

      imageops::replace(&mut fdst, &fsrc, 0, 0);
      chunk_c += 1;
    }
  }

  eprint!(" [encode]");
  canvas.save(png_out)?;

  eprintln!(" [done]");
  Ok(())
}
