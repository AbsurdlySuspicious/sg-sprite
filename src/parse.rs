use super::*;
use byteorder::{LittleEndian, ReadBytesExt};
use libflate::zlib;
use std::collections::HashMap;
use std::format as fmt;
use std::fs::File;
use std::io::{self, BufReader, Read, Seek, SeekFrom};
use SpriteT::*;

const COMMON_BUF_SZ: usize = 32;
const HEADER_SZ: usize = 4 * 2; // [u32:sprite_c][u32:chunk_c]
const SPRITE_SZ: usize = 4 * 3; // [32][u32:chunk_offset][u32:chunk_count]
const CHUNK_SZ: usize = 4 * 4; // [f32:img_x][f32:img_y][f32:chunk_x][f32:chunk_y]
const SPRITE_SIZE_ADD: i32 = 32; // dangling block
const SPRITES_MAX_RAW: u32 = 65536; // for compressed lay detection

#[derive(PartialEq, Debug)]
pub enum SpriteT {
  Base,                    // 0x00 Base sprite
  Sub,                     // 0x20 Sub sprite (implicit dep on base)
  Dep { st: u8, dep: u8 }, // 0x40,30,60 Sprite with dep on sub
  Overlay,                 // 0x50 Transparent overlay
}

#[derive(Debug)]
pub struct Sprite {
  pub t: SpriteT,
  pub id: u8,
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
  pub sub_map: HashMap<u8, usize>,
  pub chunks: Vec<Chunk>,
  pub base_dep: Option<usize>,
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
  let preread = read_u32_le(src_f)?;
  src_f.seek(SeekFrom::Start(0))?;

  if preread > SPRITES_MAX_RAW {
    eprintln!("[I] Compressed lay");
    let buf_pre = BufReader::new(src_f);
    let z = zlib::Decoder::new(buf_pre)?;
    //let buf = BufReader::new(z);
    parse_lay_impl(z)
  } else {
    eprintln!("[I] Raw lay");
    let buf = BufReader::new(src_f);
    parse_lay_impl(buf)
  }
}

fn parse_lay_impl(mut bf: impl Read) -> Result<ParsedLay, PErr> {
  let mut c_buf = [0u8; COMMON_BUF_SZ];

  let sprite_count: u32;
  let chunk_count: u32;
  {
    // read header
    let buf = &mut c_buf[..HEADER_SZ];
    bf.read_exact(buf)?;

    let buf = &mut &*buf;
    sprite_count = read_u32_le(buf)?;
    chunk_count = read_u32_le(buf)?;
  }

  let mut sprites: Vec<Sprite> = Vec::with_capacity(sprite_count as usize);
  let mut sub_map: HashMap<u8, usize> = HashMap::new();

  // read sprites
  for _i in 0..sprite_count {
    let buf = &mut c_buf[..SPRITE_SZ];
    bf.read_exact(buf)?;

    let buf = &mut &*buf;
    let mut head = [0u8; 4];
    buf.read_exact(&mut head)?;

    let type_id = head[3];
    let s = Sprite {
      t: match type_id {
        0x00 => Base,
        0x20 => Sub,
        0x40 | 0x30 | 0x60 => Dep { st: type_id, dep: head[1] },
        0x50 => Overlay,
        _ => return raise(fmt!("wrong sprite type 0x{}", hex::encode(&head[3..4]))),
      },
      id: head[0],
      c_offset: read_u32_le(buf)? as usize,
      c_count: read_u32_le(buf)? as usize,
    };

    // format warnings
    match s.t {
      Overlay => {
        //eprintln!("[W] Overlay sprite: {}", hex::encode(head.as_ref()));
        if head[1] != 0 || head[2] != 16 {
          eprintln!(
            "[W] ambiguous overlay head [1..3]: 0x{}",
            hex::encode(&head[1..3])
          )
        }
      }
      _ => {
        if head[2] != 0 {
          eprintln!(
            "[W] ambiguous sprite head [2]: 0x{}",
            hex::encode(&head[2..3])
          );
        }
      }
    }

    if s.t == Sub {
      sub_map.insert(s.id, sprites.len()); // will be occupied by index of this sprite..
    }

    sprites.push(s); // ..here
  }

  if sprites.is_empty() {
    raise("no sprites")?;
  }

  // if the base is absent - don't depend subs on anything
  let base_dep = match sprites[0].t {
    Base => Some(0),
    _ => None,
  };

  let mut chunks: Vec<Chunk> = Vec::with_capacity(chunk_count as usize);
  let mut sprite_max_x: i32 = 0;
  let mut sprite_min_x: i32 = 0;
  let mut sprite_max_y: i32 = 0;
  let mut sprite_min_y: i32 = 0;

  // read chunks
  for _i in 0..chunk_count {
    let buf = &mut c_buf[..CHUNK_SZ];
    bf.read_exact(buf)?;

    let buf = &mut &*buf;
    let mut chu = [0i32; CHUNK_SZ / 4];
    for c in &mut chu {
      *c = read_f32_le_to_i32(buf)?;
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

  /*println!(
    "sprite max/min x,y: {}, {} / {}, {}",
    sprite_max_x, sprite_max_y, sprite_min_x, sprite_min_y
  );*/

  let sprite_w = sprite_max_x + sprite_min_x.abs() + SPRITE_SIZE_ADD;
  let sprite_h = sprite_max_y + sprite_min_y.abs() + SPRITE_SIZE_ADD;

  //println!("sprite size: {}, {}", sprite_w, sprite_h);

  let res = ParsedLay {
    chunks,
    sprites,
    sub_map,
    base_dep,
    sprite_w: sprite_w as u32,
    sprite_h: sprite_h as u32,
    sprite_xy_min: (sprite_min_x, sprite_min_y),
    sprite_xy_max: (sprite_max_x, sprite_max_y),
  };

  Ok(res)
}
