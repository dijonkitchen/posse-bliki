//! The spec harness: every outcome in `spec/` has a test here (ADR 2026-05-10-spec-and-harness-over-implementation).
//!
//! One cargo integration test target, run by `make test` (`cargo test`).
//! Each module mirrors one spec area; `support/` holds the fixtures and the
//! small std-only checkers (HTML, XML, microformats, JSON Schema) that stand
//! in for the Python libraries the harness used before ADR 2026-10-11-rust-only-toolchain.

mod support;

mod aliases;
mod base_path;
mod dates;
mod design;
mod drafts;
mod frontmatter_schema;
mod html_valid;
mod idempotent;
mod indieweb_links;
mod jsonfeed;
mod links;
mod microformats;
mod performance;
mod rss;
mod scheduled;
mod supply_chain;
mod url_policy;
mod webmentions;
