use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

mod model;
mod reader;
mod texture;

use clap::Arg;
use clap::{arg, Command};
// Utility for reading in data
use reader::ArchiveCursor;

use crate::texture::Texture;

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

fn extract_archive(data: &[u8], decode: bool, out: &Path) {
    let mut archive = ArchiveCursor { data, pos: 0 };

    let version = archive.read_u32();
    if version != 2 {
        panic!("Unknown archive version {}", version);
    }

    let num_entries = archive.read_u32();

    let mut entries = vec![];
    for _ in 0..num_entries {
        let timestamp = archive.read_u32();
        let size = archive.read_u32();
        let filename_len = archive.read_u32();
        let name = archive.read_string(filename_len as usize);
        entries.push(Entry {
            timestamp,
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

        if decode {
            let lower = entry.name.to_lowercase();

            // Decompile assets
            if lower.ends_with(".btf") {
                if entry.name.starts_with("railbtm2") {
                    // Texture doesn't decode correctly, and causes garbage data in-game
                    // Assume broken file, and don't extract
                    return;
                }

                // Texture
                let tex = texture::Texture::load(slice).to_png();

                std::fs::write(out.join(format!("{}.png", &entry.name)), &tex[0]).unwrap();
            } else if lower.ends_with(".geo") {
                // Geometry
                //parse_geometry(slice, out, &entry.name);

                let model = model::Model::load(slice);

                let mtl = model.to_mtl();
                let obj = model.to_obj(&entry.name);

                std::fs::write(out.join(format!("{}.mtl", &entry.name)), &mtl).unwrap();
                std::fs::write(out.join(format!("{}.obj", &entry.name)), &obj).unwrap();
            }
        }
    }
}

fn pack_archive(out: &Path, src: &[&Path]) {
    let mut toc: Vec<Entry> = vec![];
    let mut data = vec![];

    for dir in src {
        'outer: for file in std::fs::read_dir(dir).unwrap().into_iter() {
            let path = file.unwrap().path();
            if !path.is_file() {
                continue;
            }

            let mut f = File::open(&path).expect("Failed to open source file");
            let mut raw = vec![];
            f.read_to_end(&mut raw).expect("Failed to read source file");

            let mut name = path.file_name().unwrap().to_string_lossy().to_string();
            let to_add = if name.to_lowercase().ends_with(".png") {
                name = name[0..name.len() - 4].to_string();

                let tex = Texture::from_png(&raw, 0);
                tex.to_raw()
            } else {
                raw
            };

            println!("Packed {} -> {}", &path.to_string_lossy(), name);

            for entry in &toc {
                // Don't add the same filename twice
                if entry.name == name {
                    continue 'outer;
                }
            }

            toc.push(Entry {
                timestamp: 946731600, // Year 2000 UTC
                name,
                size: to_add.len() as u32,
            });
            data.extend(to_add);
        }
    }

    let mut archive = File::create(out).expect("Unable to create archive file");
    archive.write_all(&2u32.to_le_bytes()).unwrap(); // Version
    archive
        .write_all(&(toc.len() as u32).to_le_bytes())
        .unwrap(); // Entry count

    for entry in &toc {
        archive.write_all(&entry.timestamp.to_le_bytes()).unwrap();
        archive.write_all(&entry.size.to_le_bytes()).unwrap();
        archive
            .write_all(&(entry.name.len() as u32).to_le_bytes())
            .unwrap();
        archive.write_all(&entry.name.as_bytes()).unwrap();
    }

    archive.write_all(&data).unwrap();
}

fn main() {
    let command = Command::new("redextract")
        .bin_name("redextract")
        .subcommand_required(true)
        .subcommand(
            Command::new("extract")
                .about("Extract a bgd archive")
                .arg(
                    Arg::new("raw")
                        .num_args(0)
                        .value_parser(clap::value_parser!(bool))
                        .default_missing_value("true")
                        .default_value("false")
                        .long("raw")
                        .short('r')
                        .help("Disables automatic decoding of files into modern formats"),
                )
                .arg(arg!(<archive> "Archive to extract"))
                .arg(arg!(<out> "Folder to extract to")),
        )
        .subcommand(
            Command::new("pack")
                .about("Pack a bgd archive")
                .arg(arg!(<out> "Output destination"))
                .arg(
                    Arg::new("input")
                        .help("Directories containing contents to pack. Earlier directories are chosen for conflicting files")
                        .required(true)
                        .num_args(1..),
                ),
        );

    let args = command.get_matches();
    if let Some(extract_args) = args.subcommand_matches("extract") {
        let archive_filename: &String = extract_args.get_one("archive").unwrap();
        let out_directory: &String = extract_args.get_one("out").unwrap();
        let should_decode = !*extract_args.get_one::<bool>("raw").unwrap();

        let mut file = File::open(archive_filename).expect("Failed to open archive");
        let mut raw = vec![];
        file.read_to_end(&mut raw).expect("Failed to read archive");

        let output_path = Path::new(out_directory);
        if !output_path.exists() {
            std::fs::create_dir(output_path).expect("Failed to create output directory");
        }

        extract_archive(&raw, should_decode, &output_path);
    } else if let Some(pack_args) = args.subcommand_matches("pack") {
        let archive_filename: &Path = Path::new(pack_args.get_one::<String>("out").unwrap());
        let source_directories: Vec<&Path> = pack_args
            .get_many::<String>("input")
            .unwrap()
            .map(|x| Path::new(x))
            .collect();

        for path in &source_directories {
            if !path.exists() || !path.is_dir() {
                eprintln!("Directory {} does not exist!", path.to_string_lossy());
                return;
            }
        }

        pack_archive(archive_filename, &source_directories);
    }
}
