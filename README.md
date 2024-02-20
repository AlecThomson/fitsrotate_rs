# Rusty FITSRotate

A commandline tool to rotate FITS cubes.

## Overview

This was mostly a project to help me learn Rust, but it does have a useful purpose. I've taken inspiration from [Miriad's](https://www.atnf.csiro.au/computing/software/miriad/) [reorder](https://www.atnf.csiro.au/computing/software/miriad/doc/reorder.html) command.

When reading FITS data from disk the last (FITS-ordering) axis is the slowest to read. Typically, the frequency axis is placed last. This results in slow operations along the frequency axis. One can get a nice speed up in such operations, such as RM-synthesis, by rotating the FITS cube to place the frequency axis first.

## Installation

You can install the release version using [Cargo](https://doc.rust-lang.org/cargo/):
```bash
cargo install fitsrotate_rs
```

For the latest version, you can clone this repository and build it locally:
```bash
git clone https://github.com/AlecThomson/fitsrotate_rs.git
cd fitsrotate_rs.git
cargo install --path .
```

## Usage

On the commandline:
```bash
❯ fitsrotate_rs -h
Rotate FITS images

Usage: fitsrotate_rs [OPTIONS] <FILENAME> <MODE>

Arguments:
  <FILENAME>  The FITS file
  <MODE>      Mode of rotation - a sequence of integers specifying the order of the axes (e.g. 321 for a 3D cube)

Options:
  -o, --overwrite  Overwrite the FITS file if it already exists
  -h, --help       Print help
  -V, --version    Print version
```

To use the crate in your own Rust development, add the following line to your `Cargo.toml`:
```toml
[dependencies]
fitsrotate_rs = "0.1.1"
```

## Contribution

Contributions are very welcome! As stated above, this has been a learning project for me, so please forgive any major blunders.

## License
MIT