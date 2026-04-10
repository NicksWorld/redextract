# GEO file format

The geo file format defines a 3d asset.

The file begins with a header in the following format:
| type | name | description |
| ---- | ---- | ----------- |
| char[4] | magic | Always "BGGF" |
| u32 | unk1 | Unknown |
| u32 | idx_count | Number of triangle indicies across all meshes |
| u32 | vert_count | Number of verticies across all meshes |
| u32 | mesh_count | Number of meshes contained within the model |
| u32 | unk3 | Unknown, not present when unk1 == 1 |
| f32[3][4] | bbox | Bounding boxes, first rendering, second collision |

Following the header the mesh entry block begins. The number of entries is defined
by `mesh_count` in the header. The mesh entries have the following format:
| type | name | description |
| ---- | ---- | ----------- |
| char[50] | texture | Filename of texture, null terminated |
| char[40] | name | Name of mesh, null terminated |
| u16 | unk1 | Unknown |
| u32 | color | Color attachment for shader |
| u16 | vertex_off | Offset into vertex buffer (refers to whole vertex chunk) |
| u16 | vertex_count | Number of verticies belonging to the mesh |
| u16 | index_off | Offset into index buffer (refers to individual u16 indicies) |
| u16 | index_count | Number of triangle indicies belonging to the mesh |
| u8 | render_flags | Bitfield, described below |
| char[4] | unk2 | Unknown |
| u8 | blend_mode | Valid values are 0-2, and are combined with render_flags.transparent for the final value |
| char[2] | unk3 | Unknown |

| value | name | description |
| 0x01 | Additive | Enables additive blend mode |
| 0x02 | Envmap Blend | Enables blending with the environment map. Overriden by 0x10 |
| 0x04 | Fullbright | Forces vertex colors to fullbright |
| 0x08 | Tint | Forces vertex colors to mesh color |
| 0x10 | Envmap | Sets blending to use primarily the environment map |

Following the mesh list, the index block begins. It contains triangle indicies stored
as sets of 3 u16 values. These are stored per mesh, with `index_count` sets belonging
to their respective mesh. They are stored sequentially in order of mesh entries.

Following the index block, the vertex block begins. The verticies are stored sequentially,
in the order of their respective meshes. Each vertex has the following format:
| type | name | description |
| ----- | ---- | ------- |
| f32 | x | World Coordinate X (negated) |
| f32 | y | World Coordinate Y |
| f32 | z | World Coordinate Z |
| u32 | color | Vertex Color stored in BGRA. Alpha appears unused. |
| f32 | u | Texture Coordinate U |
| f32 | v | Texture Coordinate V |
| f32 | nx | Normal X |
| f32 | ny | Normal Y |
| f32 | nz | Normal Z |
