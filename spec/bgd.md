# BGD file format

The BGD format defines an archive of uncompressed files which stores the majority
of game assets.

The format begins with two u32 values, a constant `2` followed by the number of
entries in the table of contents.

Immediately following the header, the table of contents begins. Each entry has the
following format:
| type | name | description |
| ---- | ---- | ----------- |
| u32 | timestamp | Timestamp of file creation in UTC. |
| u32 | size | Size of the described file |
| u32 | name_len | Length of filename |
| char[name_len] | name | Filename of referenced file |

Following the table of contents, the file data begins in the same order as the
table of contents entries. The size of each data block is defined by `size` within
the table of contents entry. There is no padding between files.
