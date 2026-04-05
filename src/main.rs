use std::fs::File;
use std::io::Read;
use std::io::Write;
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

fn padded_string(raw: &[u8]) -> String {
    let null_term = raw.iter().position(|&v| v == 0).unwrap_or(raw.len());
    String::from_utf8_lossy(raw.split_at(null_term).0).to_string()
}

struct Mesh {
    texture: String,
    name: String,
    index_count: u16,
    vertex_count: u16,

    index_buf: Vec<[u16; 3]>,
    vertex_buf: Vec<[f32; 9]>,
}

fn parse_geometry(raw: &[u8], name: &str) {
    let mut input = ArchiveCursor { data: raw, pos: 0 };

    let magic = input.read_slice(4);
    assert_eq!(magic, b"BGGF");

    let _unk1 = input.read_u32();
    let _idx_total = input.read_u32();
    let _unk3 = input.read_u32();
    let num_meshes = input.read_u32();
    let _unk5 = input.read_u32();

    let mut bbox = vec![];
    for _ in 0..4 {
        bbox.push([input.read_f32(), input.read_f32(), input.read_f32()]);
    }

    let mut mesh_list = vec![];
    for _ in 0..num_meshes {
        let texture = padded_string(input.read_slice(50));
        let name = padded_string(input.read_slice(40));

        let _unk_mesh_1 = input.read_u32();
        let _unk_mesh_2 = input.read_u32();

        let vertex_count = input.read_u16();
        let _unk_mesh_3 = input.read_u16();
        let index_count = input.read_u16();
        let _unk_mesh_4 = input.read_u16();

        // Might not be u16 values
        let _unk_mesh_5 = input.read_u16();
        let _unk_mesh_6 = input.read_u16();
        let _unk_mesh_7 = input.read_u16();

        mesh_list.push(Mesh {
            texture,
            name,
            index_count,
            vertex_count,

            index_buf: vec![],
            vertex_buf: vec![],
        });
    }

    for mesh in &mut mesh_list {
        for _ in 0..mesh.index_count {
            mesh.index_buf
                .push([input.read_u16(), input.read_u16(), input.read_u16()]);
        }
    }

    for mesh in &mut mesh_list {
        for _ in 0..mesh.vertex_count {
            let mut set = [0f32; 9];
            for i in 0..set.len() {
                set[i] = input.read_f32();
            }
            mesh.vertex_buf.push(set);
        }
    }

    // Janky obj output
    let mut out = File::create(format!("out/{}.obj", name)).unwrap();

    let mut vert_start: usize = 1;
    for mesh in &mesh_list {
        out.write_all(format!("g {}\n", mesh.name).as_bytes())
            .unwrap();
        for vert in &mesh.vertex_buf {
            out.write_all(format!("v {} {} {}\n", vert[0], vert[1], vert[2]).as_bytes())
                .unwrap();
        }
        for vert in &mesh.vertex_buf {
            out.write_all(format!("vt {} {}\n", vert[4], vert[5]).as_bytes())
                .unwrap();
        }

        for set in &mesh.index_buf {
            out.write_all(
                format!(
                    "f {}/{} {}/{} {}/{}\n",
                    vert_start + set[0] as usize,
                    vert_start + set[0] as usize,
                    vert_start + set[1] as usize,
                    vert_start + set[1] as usize,
                    vert_start + set[2] as usize,
                    vert_start + set[2] as usize,
                )
                .as_bytes(),
            )
            .unwrap();
        }

        vert_start += mesh.vertex_count as usize;
    }
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

        println!("Extracting {}", &entry.name);

        // Extract raw asset
        std::fs::write(out.join(&entry.name), slice).unwrap();

        // Decompile assets
        if entry.name.ends_with(".btf") {
            // Texture
            parse_image(slice, &entry.name);
        } else if entry.name.ends_with(".geo") {
            // Geometry
            parse_geometry(slice, &entry.name);
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
