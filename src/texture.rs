use std::collections::HashSet;

use image::{imageops::FilterType, GenericImageView, ImageBuffer};

use crate::reader::ArchiveCursor;

#[derive(Default)]
pub struct Texture {
    /// Image kind, only "2" is known
    kind: u16,
    /// Height of the image
    height: u16,
    /// Width of the image
    width: u16,
    /// Bits per pixel of the image data
    /// 8 implies colormap, 24 and 32 are raw RGB/RGBA
    bpp: u16,
    /// Unknown, usually 256 but some are 0
    _unk1: u16,
    /// Number of additional mip levels
    mip_count: u16,
    /// Colormap is only present with 8bpp mode
    colormap: Option<[[u8; 3]; 256]>,
    /// Image data
    image: Vec<Vec<u8>>,
}

impl Texture {
    pub fn load(raw: &[u8]) -> Texture {
        let mut input = ArchiveCursor { data: raw, pos: 0 };
        let mut tex = Texture::default();

        tex.kind = input.read_u16();
        tex.height = input.read_u16();
        tex.width = input.read_u16();
        tex.bpp = input.read_u16();
        assert_eq!(tex.bpp % 8, 0); // Ensure bpp is multiple of 8
        tex._unk1 = input.read_u16();
        tex.mip_count = input.read_u16();

        if tex._unk1 != 256 {
            // TODO: Mip handling is broken when the unknown value isn't 256
            tex.mip_count = 1;
        }

        if tex.bpp == 8 {
            let mut colormap = [[0u8; 3]; 256];
            for i in 0..colormap.len() {
                colormap[i] = [input.read_u8(), input.read_u8(), input.read_u8()];
            }
            tex.colormap = Some(colormap);
        }

        // Mipmap level dimensions
        let mut m_width = tex.width;
        let mut m_height = tex.height;

        let extract_count = if tex.mip_count == 0 { 1 } else { tex.mip_count };
        for _ in 0..extract_count {
            tex.image.push(
                input
                    .read_slice((tex.bpp / 8) as usize * m_width as usize * m_height as usize)
                    .to_owned(),
            );

            // Next mip should be exactly half-sized
            m_width /= 2;
            m_height /= 2;
        }

        // Ensure we read everything that is currently understood
        // TODO: When _unk1 is zero, we aren't reading everything. What is it?
        debug_assert!(input.pos == input.data.len() || tex._unk1 == 0);

        tex
    }

    pub fn from_png(raw: &[u8], mip_levels: u16) -> Texture {
        let mut image = image::ImageReader::new(std::io::Cursor::new(raw));
        image.set_format(image::ImageFormat::Png);
        let image = image.decode().expect("Failed to open source png");

        let mut texture = Texture {
            kind: 2,
            height: image.height() as u16,
            width: image.width() as u16,
            _unk1: 256,
            mip_count: mip_levels,
            ..Default::default()
        };

        // Determine required bpp value
        let mut color_set = HashSet::new();
        let mut has_alpha = true;
        for (_x, _y, pixel) in image.pixels() {
            if pixel[3] != 255 {
                // Alpha values require 32bpp
                has_alpha = true;
                break;
            }

            color_set.insert(pixel);
        }

        if has_alpha {
            texture.bpp = 32;
        } else if color_set.len() > 256 {
            texture.bpp = 24;
        } else {
            if mip_levels == 0 {
                //texture.bpp = 8;
                texture.bpp = 24;
            } else {
                // TODO: Support downsampling textures with colormaps
                texture.bpp = 24;
            }
        }

        // Mipmap size
        let mut m_width = image.width();
        let mut m_height = image.height();

        // Mip levels are only valid on textures with power of two dimensions
        assert!(mip_levels == 0 || (m_width.is_power_of_two() && m_height.is_power_of_two()));

        for _ in 0..(mip_levels + 1) {
            let mut out =
                vec![0u8; (texture.bpp / 8) as usize * m_width as usize * m_height as usize];
            let to_convert = image.resize(m_width, m_height, FilterType::Lanczos3);

            for (x, y, pixel) in to_convert.pixels() {
                let (real_x, real_y) = (x as usize, m_height as usize - y as usize - 1);
                let off = (texture.bpp / 8) as usize * ((real_y * m_width as usize) + real_x);

                match texture.bpp {
                    8 => {
                        todo!("Encode 8bpp textures");
                    }
                    24 => {
                        out[off] = pixel[0];
                        out[off + 1] = pixel[1];
                        out[off + 2] = pixel[2];
                    }
                    32 => {
                        out[off] = pixel[0];
                        out[off + 1] = pixel[1];
                        out[off + 2] = pixel[2];
                        out[off + 3] = pixel[3];
                    }
                    _ => unreachable!(),
                }
            }

            texture.image.push(out);

            m_width /= 2;
            m_height /= 2;
        }

        texture
    }

    /// Converts a Texture object into the btf format used by Redline
    pub fn to_raw(&self) -> Vec<u8> {
        let mut out = vec![];

        out.extend(self.kind.to_le_bytes());
        out.extend(self.height.to_le_bytes());
        out.extend(self.width.to_le_bytes());
        out.extend(self.bpp.to_le_bytes());
        out.extend(self._unk1.to_le_bytes());
        out.extend(self.mip_count.to_le_bytes());

        if let Some(colormap) = self.colormap {
            for color in colormap {
                for val in color {
                    out.extend(val.to_le_bytes());
                }
            }
        }

        for image in &self.image {
            out.extend(image);
        }

        out
    }

    /// Converts all mip levels of a Texture into png-formated images
    pub fn to_png(&self) -> Vec<Vec<u8>> {
        // Mipmap level dimensions
        let mut m_width = self.width;
        let mut m_height = self.height;

        let image_count = if self.mip_count == 0 {
            1
        } else {
            self.mip_count
        };
        let mut out = Vec::with_capacity(image_count as usize);
        for i in 0..image_count {
            let raw = &self.image[i as usize];
            let mut imagebuf = image::ImageBuffer::new(m_width as u32, m_height as u32);

            for (x, y, pixel) in imagebuf.enumerate_pixels_mut() {
                let (real_x, real_y) = (x as usize, m_height as usize - y as usize - 1);
                let off = (self.bpp / 8) as usize * ((real_y * m_width as usize) + real_x);

                match self.bpp {
                    8 => {
                        let color = self.colormap.unwrap()[raw[off] as usize];
                        *pixel = image::Rgba([color[0], color[1], color[2], 255]);
                    }
                    24 => {
                        *pixel = image::Rgba([raw[off], raw[off + 1], raw[off + 2], 255]);
                    }
                    32 => {
                        *pixel = image::Rgba([raw[off], raw[off + 1], raw[off + 2], raw[off + 3]]);
                    }
                    _ => todo!("Unknown bpp: {}", self.bpp),
                }
            }

            let mut image_raw = vec![];
            let mut writer = std::io::Cursor::new(&mut image_raw);
            imagebuf
                .write_to(&mut writer, image::ImageFormat::Png)
                .expect("Failed to encode png");

            out.push(image_raw);

            m_width /= 2;
            m_height /= 2;
        }

        out
    }
}
