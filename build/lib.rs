//! The posse-bliki build as a library: the `bliki` binary (main.rs) drives it,
//! and the test harness in `tests/` reuses its JSON and front-matter parsers.
//! Rust std only, no crates (docs/adr/0007-rust-build-zero-dependencies.md).

pub mod date;
pub mod entities;
pub mod frontmatter;
pub mod image;
pub mod json;
pub mod markdown;
pub mod mdurl;
pub mod serve;
pub mod site;
pub mod text;
pub mod unicode;
pub mod util;
pub mod webmention;
