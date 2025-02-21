use super::*;
use image::{
    self, imageops, DynamicImage, GenericImage, GenericImageView, ImageBuffer, Pixels, Rgba,
    RgbaImage, codecs::png
};
use std::format as fmt;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read};
use std::path::Path;

pub const BLOCK_W: u32 = 32;
pub const BLOCK_H: u32 = 32;
pub const CANVAS_PAD_W: u32 = 0;
pub const CANVAS_PAD_H: u32 = 0;

pub struct DrawPrep {
    img: DynamicImage,
}

pub fn draw_prep(img: &Path) -> Result<DrawPrep, SgSpriteErr> {
    eprint!("draw: decode");
    Ok(DrawPrep { img: image::open(img)? })
}

pub fn draw_sprites(
    src: &mut DrawPrep,
    png_out: impl AsRef<Path>,
    sprites: &[&Sprite],
    pass: usize,
    lay: &ParsedLay,
) -> Result<(), SgSpriteErr> {
    macro_rules! status { ($($e:tt)*) => {
    eprint!("\rdraw {:02}: {}", pass, format_args!($($e)*));
  }}
    status!("");

    let (x_mid, y_mid) = lay.sprite_xy_min;
    let x_mid = x_mid.abs();
    let y_mid = y_mid.abs();

    status!("canvas");
    let mut canvas = ImageBuffer::from_pixel(
        lay.sprite_w + CANVAS_PAD_W,
        lay.sprite_h + CANVAS_PAD_H,
        Rgba([0, 0, 0, 0]),
    );

    let mut chunk_count = 0usize;

    for s in sprites.iter().rev() {
        let chunks = &lay.chunks[s.chunk_offset..][..s.chunk_count];

        for c in chunks.iter() {
            status!("{} chunks", chunk_count + 1);

            let (cx, cy) = ((x_mid + c.img_x) as u32, (y_mid + c.img_y) as u32);
            let (sx, sy) = (c.chunk_x as u32 - 1, c.chunk_y as u32 - 1);
            let mut dst_chunk = imageops::crop(&mut canvas, cx, cy, BLOCK_W, BLOCK_H);
            let src_chunk = imageops::crop(&mut src.img, sx, sy, BLOCK_W, BLOCK_H);

            imageops::replace(&mut *dst_chunk, &*src_chunk, 0, 0);
            chunk_count += 1;
        }
    }

    eprint!(", encoding: ");
    let mut out_file = BufWriter::new(File::create(png_out)?);
    let encoder = png::PngEncoder::new_with_quality(&mut out_file, png::CompressionType::default(), png::FilterType::default());
    canvas.write_with_encoder(encoder)?;

    eprintln!("done");
    Ok(())
}
