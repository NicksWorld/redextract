use crate::reader::ArchiveCursor;

#[derive(Default, Debug)]
pub struct Model {
    _unk1: u32,
    idx_total: u32,
    vert_total: u32,
    mesh_count: u32,
    _unk2: u32,
    bbox: [[f32; 3]; 4],

    mesh_entries: Vec<MeshEntry>,

    indices: Vec<[u16; 3]>,
    vertices: Vec<Vertex>,
}

#[derive(Default, Debug)]
pub struct MeshEntry {
    texture: String,
    name: String,
    _unk1: u16,
    color: u32,
    vertex_off: u16,
    vertex_count: u16,
    index_off: u16,
    index_count: u16,
    // bitfield modifying how material is rendered. See spec/geo.md
    flags: u64,
}

#[derive(Debug)]
pub struct Vertex {
    // World coordinates
    x: f32, // negated
    y: f32,
    z: f32,
    // Stored in BGRA, alpha unused
    color: u32,
    // Texture coordinates
    u: f32,
    v: f32,
    // Normal coordinates
    nx: f32,
    ny: f32,
    nz: f32,
}

fn padded_string(raw: &[u8]) -> String {
    let null_term = raw.iter().position(|&v| v == 0);
    if let Some(pos) = null_term {
        String::from_utf8_lossy(&raw[0..pos]).to_string()
    } else {
        String::from_utf8_lossy(&raw).to_string()
    }
}

impl Model {
    pub fn load(raw: &[u8]) -> Model {
        let mut input = ArchiveCursor { data: raw, pos: 0 };
        let mut model = Model::default();

        let magic = input.read_slice(4);
        assert_eq!(magic, b"BGGF");

        // Model header
        model._unk1 = input.read_u32();
        model.idx_total = input.read_u32();
        model.vert_total = input.read_u32();
        model.mesh_count = input.read_u32();
        if model._unk1 != 1 {
            // The additional value is not present when _unk1 is 1, only when 2 or 3
            // _unk1 is 1 only for shadow models
            model._unk2 = input.read_u32();
        }

        // Bounding box?
        for i in 0..4 {
            model.bbox[i] = [input.read_f32(), input.read_f32(), input.read_f32()];
        }

        // Mesh TOC entry
        for _ in 0..model.mesh_count {
            let mut mesh = MeshEntry::default();
            mesh.texture = padded_string(input.read_slice(50));
            let name_raw = input.read_slice(40);
            assert_eq!(name_raw[39], 0);
            mesh.name = padded_string(name_raw);

            mesh._unk1 = input.read_u16();

            mesh.color = input.read_u32();

            mesh.vertex_off = input.read_u16();
            mesh.vertex_count = input.read_u16();
            mesh.index_off = input.read_u16();
            mesh.index_count = input.read_u16();

            mesh.flags = input.read_u64();

            model.mesh_entries.push(mesh)
        }

        // Indices
        for _ in 0..model.idx_total {
            model
                .indices
                .push([input.read_u16(), input.read_u16(), input.read_u16()]);
        }

        // Vertices
        for _ in 0..model.vert_total {
            model.vertices.push(Vertex {
                x: input.read_f32(),
                y: input.read_f32(),
                z: input.read_f32(),
                color: input.read_u32(),
                u: input.read_f32(),
                v: input.read_f32(),
                nx: input.read_f32(),
                ny: input.read_f32(),
                nz: input.read_f32(),
            });
        }

        model
    }

    // Produces the mtl Material definitions for the model
    pub fn to_mtl(&self) -> Vec<u8> {
        let mut out = vec![];

        // Default material used when no texture is present
        out.extend(b"newmtl _Default\n");
        for mesh in &self.mesh_entries {
            if mesh.texture.len() == 0 {
                continue;
            }

            out.extend(format!("newmtl {}\n", mesh.texture).as_bytes());
            out.extend(
                format!(
                    "map_Kd {}.btf.png\n",
                    // Strip tga extension
                    &mesh.texture[0..mesh.texture.len() - 4]
                )
                .as_bytes(),
            );
        }

        out
    }

    pub fn to_obj(&self, name: &str) -> Vec<u8> {
        let mut out = vec![];

        // Load material library
        out.extend(format!("mtllib {}.mtl\n", name).as_bytes());

        let mut vert_start: usize = 0;
        let mut index_start: usize = 0;
        for mesh in &self.mesh_entries {
            out.extend(format!("g {}\n", &mesh.name).as_bytes());

            let material = if mesh.texture.len() == 0 {
                "_Default"
            } else {
                &mesh.texture
            };
            out.extend(format!("usemtl {}\n", material).as_bytes());

            let vert_slice = &self.vertices[vert_start..vert_start + mesh.vertex_count as usize];
            let idx_slice = &self.indices[index_start..index_start + mesh.index_count as usize];

            // Vertex
            for v in vert_slice {
                // World Coordinates
                out.extend(format!("v {} {} {}\n", -v.x, v.y, v.z).as_bytes());
                // Texture Coordinates
                out.extend(format!("vt {} {}\n", v.u, v.v).as_bytes());
                // Normals
                out.extend(format!("vn {} {} {}\n", v.nx, v.ny, v.nz).as_bytes());
            }

            // Faces
            for tri in idx_slice {
                let format_vert = |x: u16| {
                    let y = 1 + vert_start + x as usize;
                    format!("{}/{}/{}", y, y, y)
                };

                out.extend(
                    format!(
                        "f {} {} {}\n",
                        format_vert(tri[0]),
                        format_vert(tri[1]),
                        format_vert(tri[2])
                    )
                    .as_bytes(),
                );
            }

            vert_start += mesh.vertex_count as usize;
            index_start += mesh.index_count as usize;
        }

        out
    }
}
