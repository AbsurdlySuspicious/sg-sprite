# sg-sprite

Sprite layout parser for MAGES. engine. 

This app restores original sprites from `.png` and `.lay` files found in `chara.mpk` archives. 
Note that this parser doesn't work with mpk files directly, you need to unpack sprites beforehand. 
There's a good tool for this: https://github.com/rdavisau/sg-unpack

Hypothetically, it should support all SciAdv series games 
on this engine (Steins;Gate Steam Edition, Steins;Gate 0, Chaos;Child, etc.), but actually it's tested
on s;g0 resources only. Compatibility list below will be updated as soon as I'll test (and maybe fix)
the parser for other titles. 
If you find out that it works with non-listed games correctly, feel free to submit PR or issue.
Or patches, if they're not. ¯\_(ツ)_/¯

You also can read format description [here](lay-format.md). 
It's based solely on reverse-engineering of s;g0 sprites and thus is rough and incomplete,
but it should give approximate vision of the file structure. 

## Missing functionality

- Doesn't draw overlays. 
  They're not very common, in the s;g0 at least.
  I'm not sure how to output them:
  Drawing each overlay on all suitable sprite variants
  will dramatically increase the count of output pngs, so
  I've just decided to not implement it until I'll come up
  with better solution.
  
## Compatibility list

- Steins;Gate 0
- Steins;Gate Steam Edition
- Steins;Gate Linear Bounded Phenogram

## Install

Prebuilts for **Windows** and **Linux** are available at 
[Github releases](https://github.com/AbsurdlySuspicious/sg-sprite/releases)

Note that this app have no gui. You should run it from console.
Run with `--help` for details on usage.

Usage example:

`sg-sprite -d out *.lay`

## Build

Install cargo (https://www.rust-lang.org/tools/install)

Run this command in the project directory: `cargo build --release`

Resulting binary will be in `target/release` directory
