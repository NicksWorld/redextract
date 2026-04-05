# BTF file format

The BTF file format provides a texture including mip maps.

The header of the format is defined as follows:
| type | name | description |
| ---- | ---- | ----------- |
| u16 | magic | Always 2 |
| u16 | height | Height of the image in pixels |
| u16 | width | Width of the image in pixels |
| u16 | bpp | Bits per pixel of image data |
| u16 | unk2 | Unknown, usually 256 - sometimes 0 |
| u16 | mips | Number of mip maps following the main image |

Following the header, if `bpp` is 8, a colormap is included.
The colormap is made up of 256 RGB values, occupying 3 bytes each.

Following the header or optional colormap, the image data is included.
The image is stored in rows of pixels (size defined by `bpp`), starting with the
bottom.

`bpp` has only 3 valid values.
| value | meaning |
| ----- | ------- |
| 8 | Index into colormap |
| 24 | Raw RGB values |
| 32 | Raw RGBA values |

The image data is stored sequentially from largest mip to smallest, halving the
dimensions with each level. The number of additional mip levels present is
defined by `mips`.
