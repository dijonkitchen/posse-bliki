# 0010 — Rust-only toolchain: harness, tables and a pinned compiler

- Date: 2026-10-11
- Status: Accepted
- Decided by: JC, asking for "a full refactor to Rust, along with env
  consistency and setup" after the build moved to Rust (ADR 0007).
- Supersedes: [0003](0003-uv-for-python-toolchain.md) (uv for Python), and
  the "Harness" section of [0007](0007-rust-build-zero-dependencies.md),
  which kept Python for the tests.

## Context

ADR 0007 replaced the Python build with a std-only Rust binary but kept
Python in two places:

- **The test harness** — 13 pytest files on five third-party packages:
  html5lib (strict HTML5 parse), mf2py (microformats), feedparser (RSS),
  PyYAML and jsonschema (front-matter reference check).
- **`scripts/gen_tables.py`** — read the HTML entity list and Unicode
  character classes out of Python's standard library to generate
  `build/entities.rs` and `build/unicode.rs`.

So a contributor still needed two toolchains (rustup and uv), CI installed
both, and five Python packages sat in `uv.lock` with their own release
cadence and supply-chain risk. The Rust compiler itself was not pinned:
ADR 0007 names 1.97, but nothing enforced it, so `make ci` could behave
differently on two machines.

## Decision

**One toolchain: Rust, pinned.**

- `rust-toolchain.toml` pins Rust 1.97.0 with rustfmt and clippy. rustup
  reads it on every machine, CI runner and cloud session, so everyone runs
  the same compiler. `make setup` installs it. We chose this over Nix: it
  pins the only tool the repo needs, rustup already handles it, and it
  adds no new system for contributors to learn.
- `make lint` (rustfmt check, clippy with warnings as errors) joins
  `make ci`, so formatting and lint rules are the same everywhere too.
  `rustfmt.toml` keeps the existing 120-column style.

**The harness is Rust.** Every pytest test now lives in `tests/harness/`,
one cargo integration-test target run by `cargo test`. It stays a black
box: it runs the `bliki` binary into temp directories and checks the files
it writes, as before. The five Python packages are replaced by small
std-only checkers in `tests/harness/support/`, each scoped to what our
output can produce and each with its own self-tests:

| Was | Now | Scope |
|---|---|---|
| html5lib strict | `html.rs` | doctype, matched tags, void elements, block-in-`<p>`, nested `<a>`, attributes, character references, control characters |
| mf2py | `mf2.rs` | root items, `p-`/`u-`/`dt-`/`e-` properties, nested items, `rels` (no implied properties: the build marks everything explicitly) |
| feedparser | `xml.rs` + RFC 822 check | XML well-formedness, channel and item fields |
| jsonschema | `schema.rs` + `regex.rs` | the keywords `spec/content-schema.json` uses; unknown keywords fail loudly |
| PyYAML | the build's own YAML parser | Rust has no YAML in std (see Consequences) |

The build becomes a library (`build/lib.rs`) plus the `bliki` binary, so
the harness can reuse its JSON parser and YAML reader.

**The table generator is Rust.** `build/gen_tables.rs` (`make tables`)
reads the published sources (WHATWG `entities.json`, the Unicode
Character Database's `UnicodeData.txt` and `DerivedNumericType.txt`)
instead of Python's standard library. The Unicode version is pinned in
the Makefile. The regenerated tables are byte-identical to the Python
output.

**Every test was checked to fail.** Breaking the output (a block element
inside `<p>`, a duplicate attribute, a missing `h-entry`, an invalid RSS
entity, a bad tag) fails the matching tests.

## Consequences

- One install (rustup) instead of two. CI no longer installs uv.
- No third-party code anywhere: not in the build, not in the tests, not in
  the tools (still ADR 0007's rule, now covering the whole repo).
- The checkers are narrower than the libraries they replace. html5lib knew
  every HTML5 parse error; `html.rs` knows the ones our templates and
  markdown renderer could hit, and is stricter in places (every element
  must be closed explicitly). If the build starts emitting new kinds of
  markup, extend the checker and its self-tests in the same change.
- The front-matter check reads YAML with the build's own parser, so it is
  no longer a fully independent reference. The schema half still is: it
  validates against `spec/content-schema.json` directly, not through the
  build's `frontmatter::validate`.
- Pytest parametrized cases became loops, so the count dropped from 128 to
  63 tests with the same assertions. A failure message names the file.
- `make tables` needs network access to download the source data. The
  build still never touches the network.
- Bumping Rust is a one-line change to `rust-toolchain.toml`. Review the
  output diff when you do, as with any other build change.

## Alternatives considered

- **Keep pytest for the harness** (ADR 0007's choice). It worked and the
  libraries are thorough, but it kept a second toolchain and five
  dependencies for code that never ships.
- **Use crates in tests only** (`scraper`, `quick-xml`, `jsonschema`,
  `serde_yaml` as dev-dependencies). Less code to write, but it reopens
  ADR 0007's dependency question, and dev-dependencies are still code we
  download and run.
- **Nix for environment consistency.** It would pin everything, but the
  repo only needs one pinned tool, and rustup already pins it from a
  checked-in file.
