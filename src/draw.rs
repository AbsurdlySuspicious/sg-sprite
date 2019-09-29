use super::*;
use crate::raise;
use image::{self, imageops, GenericImage, GenericImageView, ImageBuffer, Pixels, Rgba, RgbaImage};
use parse::*;
use std::format as fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

const BLOCK_W: u32 = 32;
const BLOCK_H: u32 = 32;
const CANVAS_PAD_W: u32 = 0;
const CANVAS_PAD_H: u32 = 0;

pub fn draw_sprite(
  png_in: &Path,
  png_out: &Path,
  sprite_idx: usize,
  lay: &ParsedLay,
) -> Result<(), PErr> {
  let sp = match lay.sprites.get(sprite_idx) {
    Some(s) => s,
    None => raise("wrong sprite index")?,
  };

  eprint!("draw {}:", sprite_idx);

  let (x_mid, y_mid) = lay.sprite_xy_min;
  let x_mid = x_mid.abs();
  let y_mid = y_mid.abs();

  let chunk_s = sp.c_offset;
  let chunk_e = chunk_s + sp.c_count;
  let chunks = &lay.chunks[chunk_s..chunk_e];

  eprint!("\rdraw {}: decode", sprite_idx);
  let mut src = image::open(png_in)?;

  eprint!("\rdraw {}: canvas", sprite_idx);
  let mut canvas = ImageBuffer::from_pixel(
    lay.sprite_w + CANVAS_PAD_W,
    lay.sprite_h + CANVAS_PAD_H,
    Rgba([0, 0, 0, 0]),
  );

  for (i, c) in chunks.iter().enumerate() {
    eprint!("\rdraw {}: chunk {}", sprite_idx, i);

    let (cx, cy) = ((x_mid + c.img_x) as u32, (y_mid + c.img_y) as u32);
    let (sx, sy) = (c.chunk_x as u32 - 1, c.chunk_y as u32 - 1);
    let mut fdst = imageops::crop(&mut canvas, cx, cy, BLOCK_W, BLOCK_H);
    let fsrc = imageops::crop(&mut src, sx, sy, BLOCK_W, BLOCK_H);

    imageops::replace(&mut fdst, &fsrc, 0, 0);
  }

  eprint!(" [encode]");
  canvas.save(png_out)?;

  eprintln!(" [done]");
  Ok(())
}
