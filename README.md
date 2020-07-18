# sg-sprite

Sprite layout parser for MAGES. engine. 

This app restores original sprites from `.png` and `.lay` files found in `chara.mpk` archives. 
Note that this parser doesn't work with mpk files directly, you need to unpack sprites beforehand. 
There's a good tool for this: https://github.com/rdavisau/sg-unpack

Compatible games are listed in the [compatibility list](#compatibility-list) below.
This list will be updated as soon as I'll test (and maybe fix) the parser for other titles. 
If you find out that it works with non-listed games correctly, feel free to submit PR or issue.
Or patches, if they're not. ¯\\_(ツ)_/¯

You also can read format description [here](lay-format.md). 
It's based solely on reverse-engineering of s;g0 sprites and thus is rough and incomplete,
but it should give approximate vision of the file structure. 

## FAQ

- My archive has .cpk extension. How do I unpack it?
- Use [arc_unpacker project](https://github.com/vn-tools/arc_unpacker):
  `arc_unpacker --dec=cri/cpk --no-recurse chara.cpk`

+ After unpacking chara archive I see `.gxt` files instead of `.png`
+ GXT is a PS Vita texture format. Convert them into png before using sg-sprite
  with this tool: [Scarlet Project](https://github.com/xdanieldzd/Scarlet).  
  Converted PNGs will have ` (Image 0)` suffix but starting from 0.2.3
  sg-sprite will pick them up nevertheless so you don't need to rename them
  to match `.lay` files names.  
  Resulting sprites may have glitchy background in this case and I suspect
  this is a gxt-to-png conversion issue (you can confirm it if you look into
  one of converted/source PNGs). Let me know if there is any
  better maintained converter so I can replace the link.

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
- Steins;Gate My Darling's Embrace
- Chaos;Child

### Non-SciAdv novels

- Yahari Game Demo Ore no Seishun Love-Kome wa Machigatteiru. Zoku

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
