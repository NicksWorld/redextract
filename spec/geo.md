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
| u32 | unk3 | Unknown |
| f32[4][3] | bbox | Bounding box? |

Following the header the mesh entry block begins. The number of entries is defined
by `mesh_count` in the header. The mesh entries have the following format:
| type | name | description |
| ---- | ---- | ----------- |
| char[50] | texture | Filename of texture, null terminated |
| char[40] | name | Name of mesh, null terminated |
| u32 | unk1 | Unknown |
| u32 | unk2 | Unknown |
| u16 | vertex_count | Number of verticies belonging to the mesh |
| u16 | unk3 | Unknown |
| u16 | index_count | Number of triangle indicies belonging to the mesh |
| u16 | unk4 | Unknown |
| u16 | unk5 | Unknown |
| u16 | unk6 | Unknown |
| u16 | unk7 | Unknown |

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
