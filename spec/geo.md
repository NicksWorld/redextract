# GEO file format

The geo file format defines a 3d asset.

The file begins with a header in the following format:
| type | name | description |
| ---- | ---- | ----------- |
| char[4] | magic | Always "BGGF" |
| u32 | unk1 | Unknown |
| u32 | idx_count | Number of triangle indicies across all meshes |
| u32 | unk2 | Unknown |
| u32 | mesh_count | Number of meshes contained within the model |
| u32 | unk3 | Unknown |
| f32[4] | bbox | Bounding box? |

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

Following the index block, the vertex block begins. It contains sets of 9 f32 values
per vertex, stored per mesh sequentially, with `vertex_count` defining the number
of vertexes belonging to their respective mesh.

The first three floats per mesh define the X, Y, and Z coordinates of the point.
The 5th and 6th floats define the UV coordinates of the vertex, used for texture mapping.
