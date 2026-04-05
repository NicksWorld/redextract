# redextract
A tool to extract and decompile assets from Redline (1999)

To extract a `bgd` archive into a directory run `redextract <archive> <out>`.
Example:
```
redextract Redline.bgd out
```

This tool can currently extract all contents successfully, though not all internal
formats are interpreted. Currently the following formats can be interpreted:
- wav audio (stored verbatim in archive)
- btf textures (automatically converted to png)
