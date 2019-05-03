# A Zathura Plugin API wrapper for Rust

[![crates.io](https://img.shields.io/crates/v/zathura-plugin.svg)](https://crates.io/crates/zathura-plugin)
[![docs.rs](https://docs.rs/zathura-plugin/badge.svg)](https://docs.rs/zathura-plugin/)
[![Build Status](https://travis-ci.org/jonas-schievink/zathura-plugin.svg?branch=master)](https://travis-ci.org/jonas-schievink/zathura-plugin)

This crate wraps [Zathura's](https://pwmt.org/projects/zathura/)
Plugin API in a memory- and typesafe Rust interface, and allows writing Zathura
plugins in Rust.

Please refer to the [changelog](CHANGELOG.md) to see what changed in the last
releases.

## Usage

Add an entry to your `Cargo.toml`:

```toml
[dependencies]
zathura-plugin = "0.4.0"
```

Check the [API Documentation](https://docs.rs/zathura-plugin/) for how to use the
crate's functionality.
