# redextract
A tool to extract and decompile assets from Redline (1999)

To extract a `bgd` archive into a directory run `redextract <archive> <out>`.
Example:
```
redextract extract Redline.bgd out
```

This tool can currently extract all contents successfully, though not all internal
formats are interpreted. Currently the following formats can be interpreted:
- wav audio (stored verbatim in archive)
- btf textures (automatically converted to png)
- geo models (automatically converted to obj)

This tool can additionally repack assets, converting some files into their native formats:
- wav audio (stored verbatim in archive)
- btf textures (converted from `*.btf.png` -> `*.btf`)

To do this, a few steps are needed
```
# Extract original Redline.bgd without decoding assets to `raw`
# It is advisible not to pack from a decoded output, as the output is not
# currently perfect.
redextract extract -r Redline.bgd raw

# After this, save any modified files to a new directory `mod`

# Pack original files alongside changes
# Any number of directories can be specified, with the earlier overriding files
# in the later.
redextract pack Redline.bgd mod raw
```
