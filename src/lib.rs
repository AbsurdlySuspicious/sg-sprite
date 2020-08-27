#![allow(dead_code, unused_imports)]

use lazy_format::lazy_format;
use std::env;
use std::fmt::Display;
use std::format as fmt;
use std::path::{Path, PathBuf};
use std::fs::{File, DirEntry};
use std::io;
use structopt::StructOpt;

mod dep;
mod draw;
mod parse;
mod util;

use dep::*;
use draw::*;
use parse::*;
use util::*;

pub type SgSpriteErr = anyhow::Error;
use anyhow::{anyhow as raise_e, bail as raise};

pub fn print_err(e: impl Display) {
    eprintln!("[E] {}", e);
}

#[derive(StructOpt, Debug)]
#[structopt()]
pub struct Opts {
    /// Output dir
    #[structopt(short, long, parse(from_os_str))]
    pub dir: Option<PathBuf>,

    /// Limit variants to draw per sprite
    #[structopt(short, long)]
    pub limit: Option<usize>,

    /// .lay files to parse
    #[structopt(name = "LAY_FILES", parse(from_os_str))]
    pub lay_files: Vec<PathBuf>,

    /// Perform parsing only to test for errors.
    /// Do not compose actual images
    #[structopt(long)]
    pub dry_run: bool,
}

const LAY_EXT: &[&str] = &["_.lay", ".lay"];

pub fn lib_main(o: &Opts) -> Result<(), SgSpriteErr> {
    let layouts = &o.lay_files;
    let out_dir = o.dir.as_ref();

    match out_dir {
        Some(d) if !d.is_dir() => raise!("out_dir isn't a directory"),
        None if !o.dry_run => {
            raise!("Output dir should be specified (-d)\nSee --help for details");
        }
        _ => (),
    }

    let total = o.lay_files.len();

    if total == 0 {
        raise!("no .lay files provided");
    }

    let status = |c: usize| move |t: &str| println!("[{}/{}] {}", c + 1, total, t);

    for i in 0..layouts.len() {
        let lay_path = &layouts[i];
        if let Err(e) = lay_in(out_dir, lay_path, o.limit, status(i)) {
            print_err(e);
            print_err(format_args!("({})", lay_path.display()));
        }
    }

    Ok(())
}

fn lay_in(
    out_dir: Option<impl AsRef<Path>>,
    lay_file: &Path,
    limit: Option<usize>,
    status_cb: impl Fn(&str),
) -> Result<(), SgSpriteErr> {
    let limit = limit.unwrap_or(0);

    let (lay_filename, lay_ext) = lay_file
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|f| LAY_EXT.iter().find(|e| f.ends_with(**e)).map(|e| (f, e)))
        .ok_or_else(|| raise_e!("not a lay file"))?;

    let sprite_name =
        lay_filename.trim_end_matches(lay_ext);

    status_cb(sprite_name);

    let lay = parse_lay(&mut File::open(lay_file)?)?;
    let graph = DepGraph::resolve_dep_graph(&lay);
    let leaves = graph.get_leaf_sprites();

    if let Some(out_dir) = out_dir {
        let src_image_path: PathBuf = {
            let mut path_buf = lay_file.canonicalize().expect("Can't canonicalize .lay path");
            let parent_dir = path_buf.parent().expect("No parent dir");
            // bail out right away if there are problems with path resolution

            let src_filename = parent_dir.read_dir()
                .unwrap()
                .flatten()
                .find(|f|
                    f.file_name().to_str()
                        .map(|n| n.starts_with(sprite_name) && n.ends_with(".png"))
                        .unwrap_or(false)
                )
                .ok_or_else(|| raise_e!("No corresponding png file"))?;

            path_buf.pop();
            path_buf.push(src_filename.file_name());
            path_buf
        };

        println!("[I] Using source file: {}",
                 src_image_path.file_name().unwrap().to_str().unwrap_or("???"));

        let mut src_image = draw_prep(&src_image_path)?;

        for (pass, sp) in leaves.enumerate() {
            if limit > 0 && pass >= limit {
                eprintln!("[I] Limit reached, proceeding to next sprite");
                break;
            }

            let s = sp.sprite;
            let name_suf = lazy_format!(match (s.sprite_type) {
                SpriteT::Base => ("b{}", s.id),
                SpriteT::Sub => ("s{}", s.id),
                SpriteT::Overlay => ("o{}", s.id),
                SpriteT::Dep { exact_type: st, depends_on: dep } => ("t{}_d{}_{}", st, dep, s.id),
            });

            let layers = graph.resolve_layers(sp)?;
            let mut out_path = out_dir.as_ref().to_path_buf();
            out_path.push(fmt!("{}_{}.png", sprite_name, name_suf));

            if let Err(e) = draw_sprites(&mut src_image, &out_path, layers.as_ref(), pass + 1, &lay) {
                print_err(e);
            }
        }
    } else {
        for sp in leaves {
            graph.resolve_layers(sp)?; // validation resolve for --dry-run
        }
    }

    Ok(())
}
