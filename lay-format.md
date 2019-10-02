# Legend

`[x:y]` - single logical unit of binary file, where:
- `x` - size
- `y` - name

Sizes: `x00` (`u32`, `u8`, `f32`, etc)
- `x` - type
  - `u` - unsigned integer
  - `i` - signed integer
  - `f` - float
  - if no type specified, this is a subsection that will be explained later
- `00` - size in bits
  
  
# File

.lay consists of 3 major sections, going one by one:
- header
- sprite list
- chunk list
- some optional junk at the end of file

Sprite list defines variants of this sprite. They have several types and can be dependent on each other.

Chunk list defines positions of chunks in the source png and their position in the target sprite.

Source file chunks are 32x32px regions in the source png composed into the final sprite
according to the needed sprite variant.

Note: all values in the file are LE 

# Header

Whole header (8 bytes):

`[u32:sprite_count][u32:chunk_count]`

- `sprite_count` - count of sprite entries in the corresponding section
- `chunk_count` - count of chunk entries

All these counts represents total count of corresponding entries in the whole file.

# Sprite list

Sprite entry (12 bytes per one):

`[32:info][u32:chunk_offset][u32:chunk_count]`

- `chunk_offset` - index of first chunk entry of this sprite variant
- `chunk_count`  - number of chunks to draw

Info:

Info part is consists of 4 bytes, which meaning depends on the variant type, so we'll give them names:

`[8:A][8:B][8:C][8:D]`

`D` is always a type indicator.

Types:

- `D = 0x00` - Base sprite, always drawn first if exists
    - `A` - Sprite id. For base sprite it's usually `0x01`. 
            Base sprite id isn't actually used anywhere and probably can be ignored.
	    Same appliable for other sprite's ids unless stated otherwise.
    - `B, C` - always `0x00` 
- `D = 0x20` - Sub sprite. Usually different faces for the base sprite. 
               Implicitly depends on the base sprite if exists
    - `A` - Sprite id. Used later for dependencies in `Dep` sprites
    - `B, C` - always `0x00`
- `D = 0x40` - Dep sprite. Usually mouths for lipsync.
    - `A` - Sprite id.
    - `B` - Id of sub sprite which this sprite depends on.
            Absence of sub sprite with this id implies that 
            this sprite depends directly on base or, if base
            is absent too, doesn't depends on anything.
    - `C` - always `0x00`
- `D = 0x50` - Overlay. Should be drawn on top of anything else with blending.
               Doesn't define dependence on any other sprites, but obviously should be drawn last.
    - `A` - Sprite id.
    - `B` - always `0x00`
    - `C` - `0x10` (purpose unknown)

Shortly, dependence chain can be expressed like this:

```
`Dep(B)` -->  if `Sub` with id `B` exists,
              depend on it  |
      _______/              |
     /                      |[else]
    /                       |
   v                        v
`Sub`    -->  if `Base` exists,
              depend on it  |
       ______/              |
      /                     |[else]
     /                      |
    v                       v
`Base`   -->  draw this first
```

Sprites should be drawn in reverse-dependence order on top of each other:

`Base -> Sub -> Dep [-> Overlay]`

Overlays are optional and usually appliable on top of any combination of sprites,
so if certian overlay needed, it should be drawn last.

Overlays must be drawn with blending (they're contains significant alpha channel).
Other sprite types doesn't need to be blended and can be drawn over canvas using simple byte replace.

# Chunk list

To draw sprites on the canvas, you should follow offset-count pair from the sprite entry above
and draw chunks one by one.

Chunk entry (16 bytes per one):

`[f32:dst_x][f32:dst_y][f32:src_x][f32:src_y]`

Coordinates are encoded as floats, but they never have fract part, so they can be safely converted to integer.

- `src_x`, `src_y` - Chunk position in the source png.
                     Those coords are absolute and never negative.
                     They're points to the (1,1) pixel of the chunk (not 0,0), so
                     they should be subtracted by 1 before using.
                     All chunks are assumed to be 32x32 px in size.
- `dst_x`, `dst_y` - Position of chunk on the target sprite/canvas.
                     They're pointing exactly to upper-left corner (0,0) 
                     of target chunk, so they shouldn't be subtracted.
                     The 0,0 point for those coords is located at the center of screen, so
                     they can be negative. If you want to draw the full sprite
                     (not to fit it into some screen) - you should initially find
                     min and max values of them to figure out the real size of target sprite.
                     
