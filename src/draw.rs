use super::*;
use crate::raise;
use image::{
  self, imageops, DynamicImage, GenericImage, GenericImageView, ImageBuffer, Pixels, Rgba,
  RgbaImage,
};
use parse::*;
use std::format as fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub const ENABLE_DRAWING: bool = true;

pub const BLOCK_W: u32 = 32;
pub const BLOCK_H: u32 = 32;
pub const CANVAS_PAD_W: u32 = 0;
pub const CANVAS_PAD_H: u32 = 0;

pub struct DrawPrep {
  img: DynamicImage,
}

pub fn draw_prep(img: &Path) -> Result<DrawPrep, PErr> {
  let img = if ENABLE_DRAWING {
    eprint!("draw: decode ({})", img.display());
    image::open(img)?
  } else {
    DynamicImage::new_rgba8(0, 0)
  };
  Ok(DrawPrep { img })
}

pub fn draw_sprites(
  src: &mut DrawPrep,
  png_out: &Path,
  sprites_rev: &[&Sprite],
  pass: usize,
  lay: &ParsedLay,
) -> Result<(), PErr> {
  if !ENABLE_DRAWING {
    return Ok(());
  }
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
      let fsrc = imageops::crop(&mut src.img, sx, sy, BLOCK_W, BLOCK_H);

      imageops::replace(&mut fdst, &fsrc, 0, 0);
      chunk_c += 1;
    }
  }

  eprint!(" [encode]");
  canvas.save(png_out)?;

  eprintln!(" [done]");
  Ok(())
}
