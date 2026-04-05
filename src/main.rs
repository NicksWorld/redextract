use std::fs::File;
use std::io::Read;
use std::path::Path;

mod reader;

use clap::{arg, Command};
// Utility for reading in data
use reader::ArchiveCursor;

// Archive TOC Entry
#[derive(Debug)]
struct Entry {
    // Modification timestamp? Always within 1999 UTC
    timestamp: u32,
    // Length of data segment
    size: u32,
    // Filename
    name: String,
}

fn parse_image(raw: &[u8], name: &str) {
    let mut input = ArchiveCursor { data: raw, pos: 0 };

    let kind = input.read_u16();
    assert_eq!(kind, 2); // Others unknown

    let height = input.read_u16();
    let width = input.read_u16();

    let bpp = input.read_u16();
    assert!(bpp % 8 == 0);
    let bytes_per_pixel = bpp / 8;

    let _unk1 = input.read_u16(); // Usually 256
    let mip_count = input.read_u16(); // Mip levels

    // Colormap (only when 8bpp)
    let mut colormap = vec![];
    if bpp == 8 {
        for _ in 0..256 {
            colormap.push((input.read_u8(), input.read_u8(), input.read_u8()));
        }
    }

    let mut mip_width = width;
    let mut mip_height = height;
    let mip_count = if mip_count == 0 || _unk1 != 256 {
        1
    } else {
        mip_count
    };
    for mip in 0..mip_count {
        let img =
            input.read_slice(bytes_per_pixel as usize * mip_width as usize * mip_height as usize);

        // Create png
        let mut imagebuf = image::ImageBuffer::new(mip_width as u32, mip_height as u32);

        for (x, y, pixel) in imagebuf.enumerate_pixels_mut() {
            // Determine pixel index
            let (rx, ry) = (x, mip_height as u32 - y - 1);
            // Get exact offset of pixel data
            let offset =
                bytes_per_pixel as usize * ((ry as usize * mip_width as usize) + rx as usize);
            // Determine pixel color
            if bytes_per_pixel == 1 {
                // Paletted
                let color =
                    &colormap[img[(ry as usize * mip_width as usize) + rx as usize] as usize];
                *pixel = image::Rgba([color.0, color.1, color.2, 255]);
            } else if bytes_per_pixel == 3 {
                // RGB one byte per channel
                *pixel = image::Rgba([img[offset], img[offset + 1], img[offset + 2], 255]);
            } else if bytes_per_pixel == 4 {
                // RGBA one byte per channel
                *pixel = image::Rgba([
                    img[offset],
                    img[offset + 1],
                    img[offset + 2],
                    img[offset + 3],
                ]);
            } else {
                todo!();
            }
        }

        let mip_suffix = if mip == 0 {
            String::from("")
        } else {
            format!(".{}", mip)
        };
        if mip == 0 {
            // Only save first mip. Eventually should be optional
            imagebuf
                .save(format!("out/{}{}.png", name, mip_suffix))
                .unwrap();
        }

        mip_width /= 2;
        mip_height /= 2;
    }

    // TODO: When _unk1 is 0, we don't read everything.
    // What is the remainder?
    assert!(input.pos == input.data.len() || _unk1 == 0);
}

fn extract_archive(data: &[u8], out: &Path) {
    let mut archive = ArchiveCursor { data, pos: 0 };

    let version = archive.read_u32();
    if version != 2 {
        panic!("Unknown archive version {}", version);
    }

    let num_entries = archive.read_u32();

    let mut entries = vec![];
    for _ in 0..num_entries {
        let unknown = archive.read_u32();
        let size = archive.read_u32();
        let filename_len = archive.read_u32();
        let name = archive.read_string(filename_len as usize);
        entries.push(Entry {
            timestamp: unknown,
            size,
            name,
        });
    }

    // Extract data to disk
    for entry in entries {
        let slice = archive.read_slice(entry.size as usize);

        // Extract raw asset
        std::fs::write(out.join(&entry.name), slice).unwrap();

        // Decompile assets
        if entry.name.ends_with(".btf") {
            // Texture
            parse_image(slice, &entry.name);
        }
    }
}

fn main() {
    let command = Command::new("redextract")
        .bin_name("redextract")
        .subcommand_required(true)
        .subcommand(
            Command::new("extract")
                .about("Extract an bgd archive")
                .arg(arg!(<archive> "Archive to extract"))
                .arg(arg!(<out> "Folder to extract to")),
        );

    let args = command.get_matches();
    if let Some(extract_args) = args.subcommand_matches("extract") {
        let archive_filename: &String = extract_args.get_one("archive").unwrap();
        let out_directory: &String = extract_args.get_one("out").unwrap();

        let mut file = File::open(archive_filename).expect("Failed to open archive");
        let mut raw = vec![];
        file.read_to_end(&mut raw).expect("Failed to read archive");

        let output_path = Path::new(out_directory);
        if !output_path.exists() {
            std::fs::create_dir(output_path).expect("Failed to create output directory");
        }

        extract_archive(&raw, &output_path);
    }
}
