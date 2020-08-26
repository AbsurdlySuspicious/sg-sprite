use sg_sprite::*;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::collections::HashMap;
use std::fs::{File, remove_dir_all, create_dir};
use std::io::{self, BufReader, BufRead};
use sha2::{Digest, Sha256};
use data_encoding::HEXLOWER;
use sha2::digest::FixedOutput;

const FILES_DIR: &str = "./tests/repo/files/";
const TEST_OUT: &str = "./target/test_out/";

#[test]
#[ignore]
fn validate_output() {
    let lay_ext = OsStr::new("lay");
    let png_ext = OsStr::new("png");
    let out_dir = Path::new(TEST_OUT);

    if out_dir.exists() {
        remove_dir_all(out_dir).unwrap();
    }

    create_dir(out_dir).unwrap();

    let ref_hashes_file: PathBuf =
        [FILES_DIR, "reference.sha256sums"].iter().collect();
    let lay_files: Vec<_> = Path::new(FILES_DIR)
        .read_dir().unwrap()
        .flat_map(|r| r.ok().map(|f| f.path()))
        .filter(|f| f.extension() == Some(lay_ext))
        .collect();

    let mut ref_hashes = HashMap::new();
    for line in BufReader::new(File::open(&ref_hashes_file).unwrap()).lines().map(|r| r.unwrap()) {
        let mut sp = line.splitn(2, "  ");
        let (hash, file) = (sp.next().unwrap(), sp.next().unwrap());
        ref_hashes.insert(file.to_string(), hash.to_string());
    }

    let o = Opts {
        dir: Some(out_dir.to_path_buf()),
        limit: None,
        lay_files,
        dry_run: false
    };

    lib_main(&o).unwrap();

    for f in out_dir.read_dir().unwrap().map(|r| r.unwrap()) {
        let path = f.path();
        if path.extension() != Some(png_ext) { continue }

        let png_name = path.file_name().and_then(|n| n.to_str()).unwrap();
        println!("Testing {}", png_name);

        let ref_hash_hex = match ref_hashes.remove(png_name) {
            Some(hash) => hash,
            None => panic!("No reference hash for {}", png_name),
        };

        let mut hasher = Sha256::new();
        let mut reader = BufReader::new(File::open(&path).unwrap());
        io::copy(&mut reader, &mut hasher).unwrap();

        let chk_hash = hasher.finalize_fixed();
        let mut ref_hash = [0_u8; 256 / 8];
        HEXLOWER.decode_mut(ref_hash_hex.as_bytes(), &mut ref_hash).unwrap();

        let chk_hash_slice: &[u8] = chk_hash.as_ref();
        assert_eq!(chk_hash_slice, ref_hash.as_ref(), "hash {}", png_name);
    }

    if !ref_hashes.is_empty() {
        println!("These reference hashes left unchecked:");
        ref_hashes.iter().for_each(|(f, h)| println!("{}  {}", h, f));
        panic!("Some reference hashes left ({})", ref_hashes.len());
    }
}
