use super::*;
use byteorder::{LittleEndian, ReadBytesExt};
use std::format as fmt;
use std::fs::File;
use std::io::{self, BufReader, Read};

const COMMON_BUF_SZ: usize = 32;
const HEADER_SZ: usize = 4 * 2; // [u32:sprite_c][u32:chunk_c]
const SPRITE_SZ: usize = 4 * 3; // [32][u32:chunk_offset][u32:chunk_count]
const CHUNK_SZ: usize = 4 * 4; // [f32:img_x][f32:img_y][f32:chunk_x][f32:chunk_y]
const SPRITE_SIZE_ADD: i32 = 32; // dangling block

#[derive(Debug)]
pub struct Sprite {
  //pre: [u8; 4],
  pub c_offset: usize,
  pub c_count: usize,
}

#[derive(Debug)]
pub struct Chunk {
  pub img_x: i32,
  pub img_y: i32,
  pub chunk_x: i32,
  pub chunk_y: i32,
}

pub struct ParsedLay {
  pub sprites: Vec<Sprite>,
  pub chunks: Vec<Chunk>,
  pub sprite_w: u32,
  pub sprite_h: u32,
  pub sprite_xy_min: (i32, i32),
  pub sprite_xy_max: (i32, i32),
}

#[inline]
fn read_u32_le(src: &mut impl Read) -> io::Result<u32> {
  src.read_u32::<LittleEndian>()
}

#[inline]
fn read_f32_le_to_i32(src: &mut impl Read) -> Result<i32, PErr> {
  let f = src.read_f32::<LittleEndian>()?;
  if f.is_nan() || f.is_infinite() {
    raise(fmt!("unsuitable f32 {}", f))?
  }
  if f.fract() != 0f32 {
    raise(fmt!("f32 has fract part {}", f))?
  }
  Ok(f as i32)
}

pub fn parse_lay(src_f: &mut File) -> Result<ParsedLay, PErr> {
  let mut c_buf = [0u8; COMMON_BUF_SZ];
  let mut bf = BufReader::new(src_f);

  let sprite_count: u32;
  let chunk_count: u32;
  {
    // read header
    let buf = &mut c_buf[..HEADER_SZ];
    bf.read_exact(buf)?;

    let buf = &mut buf.as_ref();
    sprite_count = read_u32_le(buf)?;
    chunk_count = read_u32_le(buf)?;
  }

  let mut sprites: Vec<Sprite> = Vec::with_capacity(sprite_count as usize);

  // read sprites
  for _i in 0..sprite_count {
    let buf = &mut c_buf[..SPRITE_SZ];
    bf.read_exact(buf)?;

    let buf = &mut buf.as_ref();
    read_u32_le(buf)?; // discard pre

    let s = Sprite {
      c_offset: read_u32_le(buf)? as usize,
      c_count: read_u32_le(buf)? as usize,
    };

    sprites.push(s);
  }

  let mut chunks: Vec<Chunk> = Vec::with_capacity(chunk_count as usize);
  let mut sprite_max_x: i32 = 0;
  let mut sprite_min_x: i32 = 0;
  let mut sprite_max_y: i32 = 0;
  let mut sprite_min_y: i32 = 0;

  // read chunks
  for _i in 0..chunk_count {
    let buf = &mut c_buf[..CHUNK_SZ];
    bf.read_exact(buf)?;

    let buf = &mut buf.as_ref();
    let mut chu = [0i32; CHUNK_SZ / 4];
    for i in 0..(CHUNK_SZ / 4) {
      chu[i] = read_f32_le_to_i32(buf)?;
    }

    let (img_x, img_y) = (chu[0], chu[1]);
    sprite_max_x = sprite_max_x.max(img_x);
    sprite_min_x = sprite_min_x.min(img_x);
    sprite_max_y = sprite_max_y.max(img_y);
    sprite_min_y = sprite_min_y.min(img_y);

    let s = Chunk {
      img_x,
      img_y,
      chunk_x: chu[2],
      chunk_y: chu[3],
    };

    chunks.push(s);
  }

  println!(
    "sprite max/min x,y: {}, {} / {}, {}",
    sprite_max_x, sprite_max_y, sprite_min_x, sprite_min_y
  );

  let sprite_w = sprite_max_x + sprite_min_x.abs() + SPRITE_SIZE_ADD;
  let sprite_h = sprite_max_y + sprite_min_y.abs() + SPRITE_SIZE_ADD;

  println!("sprite size: {}, {}", sprite_w, sprite_h);

  if sprite_count > 1 {
    //raise("sprite_c > 1 unimplemented")?
  }

  if sprites.is_empty() {
    raise("no sprites")?;
  }

  let res = ParsedLay {
    chunks,
    sprites,
    sprite_w: sprite_w as u32,
    sprite_h: sprite_h as u32,
    sprite_xy_min: (sprite_min_x, sprite_min_y),
    sprite_xy_max: (sprite_max_x, sprite_max_y),
  };

  Ok(res)
}
