# sg-sprite

Sprite layout parser for MAGES engine. 

This app restores original (full and uncropped) sprites from `.png` and `.lay` files found in `chara.mpk` archives. 
Note that this parser doesn't work with mpk files directly, you need to unpack sprites beforehand. 
There's a good tool for this: https://github.com/rdavisau/sg-unpack

Compatible games are listed in the [compatibility list](#compatibility-list) below.
This list will be updated as soon as I or someone else test (and fix if needed and possible) the parser for other titles. 
If you find out that it works (or doesn't work) with non-listed games built on mages engine, feel free to submit PR or issue.

You also can read format description [here](lay-format.md). 
It's based solely on reverse-engineering of s;g0 sprites and thus is rough and incomplete,
but it should give approximate vision of the file structure. 

## FAQ

-
    - **Q:** My archive has .cpk extension. How do I unpack it?
    - **A:** Use [arc_unpacker project](https://github.com/vn-tools/arc_unpacker):
      `arc_unpacker --dec=cri/cpk --no-recurse chara.cpk` (it can extract mpk files too: `--dec=mages/mpk`)
-
    - **Q:** After unpacking chara archive I see `.gxt` files instead of `.png`
    - **A:** GXT is a PS Vita texture format. Convert them into png before using sg-sprite
      with this tool: [Scarlet Project](https://github.com/xdanieldzd/Scarlet).  
      Converted PNGs will have ` (Image 0)` suffix but starting from 0.2.3
      sg-sprite will pick them up too, so you should't need to rename them
      to match `.lay` files names.  
      Resulting sprites may have glitchy background in this case, and I suspect
      this is a gxt-to-png conversion issue (you can confirm it if you look into
      one of converted/source PNGs). Let me know if there is any
      better maintained converter so I can replace the link.
-
    - **Q:** I see some transparent PNGs with `_oX` suffix in output folder. What's this?
    - **A:** These are overlays. They are intended to be drawn on top of the sprite, 
      you can do this yourself in your favorite photo editor (e.g. GIMP). 
      They should be compatible with most of the sprites in file. Also, they have
      same size as original sprite, so you don't need to do any manual positioning.
  
## Compatibility list

These titles are known to work, but others might work too (see top of readme)

- Steins;Gate 0
- Steins;Gate Steam Edition
- Steins;Gate Linear Bounded Phenogram
- Steins;Gate My Darling's Embrace
- Chaos;Child
- Yahari Game Demo Ore no Seishun Love-Kome wa Machigatteiru. Zoku

## Install

Builds for **Windows** and **Linux** are available at 
[Github releases](https://github.com/AbsurdlySuspicious/sg-sprite/releases)

There's also [AUR package](https://aur.archlinux.org/packages/sg-sprite/) available for Arch Linux.

Note that this app has no gui. You should run it from terminal (or cmd.exe on windows).
Run `sg-spirte --help` for details on usage.

Usage example:

- `cd` to directory with unpacked sprites
- create output directory (e.g. `out`)
- run `sg-sprite -d out *.lay`

## Build

Install cargo (https://www.rust-lang.org/tools/install)

Run this command in the project directory: `cargo build --release`

Resulting binary will be in `target/release` directory
