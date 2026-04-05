use image::ImageBuffer;

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

    /// Converts all mip levels of a Texture into png-formated images
    pub fn to_image(&self) -> Vec<Vec<u8>> {
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
