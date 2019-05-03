//! Raw FFI bindings to [Zathura's] plugin API.
//!
//! For more high-level bindings that allow writing plugins in type-safe Rust,
//! see [`zathura-plugin`].
//!
//! [Zathura's]: https://pwmt.org/projects/zathura/
//! [`zathura-plugin`]: https://docs.rs/zathura-plugin/

#![allow(nonstandard_style)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
