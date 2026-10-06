# 0007 — Rust build with zero dependencies

- Date: 2026-10-06
- Status: Accepted
- Decided by: JC, choosing this over a Python standard-library build
  (PR #4, closed) after comparing the two side by side.

## Context

[ADR 0001](0001-spec-and-harness-over-implementation.md) makes the build
replaceable: `spec/` and `tests/` are the durable artifacts, and any
implementation that keeps the harness green is acceptable.

The first implementation was about 530 lines of Python on four runtime
dependencies: markdown-it-py, Jinja2, PyYAML and jsonschema. Their
transitive packages added markupsafe, mdurl, mdit-py-plugins, referencing,
rpds-py and attrs, and `make serve` added watchfiles. Each one is a release
cadence, a supply-chain risk and a chance for output to shift between
versions. That works against a site meant to last ten years with
byte-stable output, which is what
[build-invariants](../../spec/build-invariants.md) promises.

Two replacements were built and compared, each producing the same site:

| | Python, stdlib only (#4) | Rust, std only (#6, this ADR) |
|---|---|---|
| Third-party packages at build time | 0 | 0 |
| Build code | ~2,900 lines | ~8,500 lines (~4,100 hand-written, ~4,400 generated tables) |
| Build of this vault | ~160 ms | ~20 ms |
| Build needs | Python 3.13 interpreter | cargo once, then a 1.0 MiB static binary |
| Output vs. old build | byte-identical | byte-identical |
| Harness | 96 tests pass | 96 tests pass, plus 10 Rust unit tests |

## Decision

The build is a Rust program that uses only the standard library.

### Shape

- `Cargo.toml` at the repo root declares one binary, `bliki`, with
  `path = "build/main.rs"` and an empty `[dependencies]` table.
  `Cargo.lock` is committed and lists exactly one package, this one.
  Adding a crate needs a new ADR that supersedes this one.
- The release profile uses `lto = true`, `codegen-units = 1` and
  `strip = true`, which produces a 1.0 MiB binary.
- It was built and tested with Rust 1.97 (edition 2021).

### Modules (`build/`)

| File | Lines | Role |
|---|---|---|
| `main.rs` | 117 | CLI: `--content`, `--out`, `--print-config`, `--serve`, `--port` |
| `site.rs` | 600 | Loading notes, URLs, backlinks, tags, aliases, feeds, sitemap; the former Jinja templates as functions |
| `markdown.rs` | 2,877 | Port of markdown-it-py 4.2's `gfm-like` preset (`html=False`, `linkify=False`, `typographer=False`) |
| `mdurl.rs` | 596 | markdown-it's link normalisation: mdurl parse/format/encode/decode and RFC 3492 punycode |
| `frontmatter.rs` | 1,174 | YAML subset parser and a hand-written check of `spec/content-schema.json` |
| `text.rs` | 255 | Wikilinks, inline `#tags`, slugify, HTML stripping and `href` scanning (Python regex semantics) |
| `date.rs` | 114 | Calendar maths for RFC 822 (RSS) and RFC 3339 (JSON Feed, sitemap) dates |
| `json.rs` | 104 | JSON matching Python's `json.dumps(indent=2)`, including `ensure_ascii` escapes |
| `serve.rs` | 168 | `std::net` static file server that polls content mtimes and rebuilds |
| `util.rs` | 99 | HTML escaping (Jinja autoescape and markdown-it variants) and Python string helpers (`strip`, `split`, `repr`) |
| `entities.rs` | 2,134 | Generated: HTML5 named character references |
| `unicode.rs` | 278 | Generated: Unicode punctuation/symbol and word-character ranges |

All string indexing works on Unicode code points (`Vec<char>`), so slicing
and offsets behave as they did in Python.

### Generated tables

`entities.rs` and `unicode.rs` are data, not hand-written code. They come
from Python's `html.entities` and `unicodedata` via
`scripts/gen_tables.py` (`make tables`). With Python 3.13.16 (Unicode
15.1.0), the version named in `unicode.rs`'s header, the script reproduces
both files byte for byte. Running it under a newer Python updates the
Unicode version. Review that diff like any other output change.

### Harness

Python stays, but only as the test harness. `uv run pytest` drives the
binary as a subprocess: `tests/conftest.py` builds it once per session,
reads the site config from `bliki --print-config`, and runs
`bliki --content … --out …`. Test assertions are unchanged. Failed builds
surface `build error: …` on stderr with exit code 1.

`pyproject.toml` has `dependencies = []`. PyYAML and jsonschema move to the
dev group because `tests/test_frontmatter_schema.py` uses them as an
independent reference check of every note's front matter. They are never
part of the build.

`make test` runs `cargo test` and then pytest. `make ci` is `sync test build`,
as before, and CI needs no workflow changes because `ubuntu-latest` ships
with cargo.

### How equivalence was established

- `diff -r` of the old Python build's output against the Rust build's
  output, over all of `content/`, shows no differences. The only intended
  change is the colophon's description of the stack.
- The same check passes on the vault plus the 21 posts imported from
  Substack in PR #5 (68 notes, 142 pages).
- Differential fuzzing of `markdown.rs` against markdown-it-py on 24,000
  generated adversarial documents found 0 mismatches. Fuzzing of the text
  helpers (wikilinks, tags, slugify, HTML stripping, href scanning) on
  30,000 inputs also found 0 mismatches.
- Two consecutive builds produce identical trees, which
  `tests/test_idempotent.py` also checks.

## Consequences

- There is nothing to pin, audit or chase for CVEs at build time, and
  the lockfile cannot drift.
- Deployment is a single static binary with no interpreter or virtualenv.
  Contributors who only edit `content/` need nothing new.
- A build of this vault takes about 20 ms, which leaves a lot of headroom
  under the "1,000 posts in under 5 seconds" invariant.
- We own about 4,100 lines of hand-written Rust, mostly the Markdown parser.
  It is a port of a specified format rather than business logic, and
  the harness pins its behaviour, but its bugs are ours to fix.
- Changing the build needs a Rust toolchain.
- Markdown behaviour is frozen at markdown-it-py 4.2 `gfm-like`. Changing
  it, for example to add footnotes or heading anchors, means changing
  `markdown.rs` and the spec and tests together, following the normal
  feature loop in `AGENTS.md`.
- The decision is reversible. The harness defines correctness, so a future
  ADR can replace this implementation with anything that keeps it green.
  The Python standard-library version in PR #4 is a ready fallback.

## Alternatives considered

- **Python, standard library only** (PR #4). It needs about a third of the
  code and keeps one language in the repo. It was not chosen because it
  still needs an interpreter at build time and is about 8× slower. JC
  preferred a build artifact with no runtime at all.
- **Keep the four Python dependencies.** This is the least work today, but
  the dependency surface is the main long-term risk to byte-stable output.
- **Rust with crates** (`pulldown-cmark`, `serde_yaml`, a template engine).
  This would be far less code, but `pulldown-cmark` does not reproduce
  markdown-it's output, so existing pages would change, and it brings back
  the dependency tree this ADR removes.
- **An off-the-shelf SSG** (Hugo, Zola, Quartz). Already rejected by
  [ADR 0002](0002-no-quartz-no-vendored-ssg.md), and nothing here changes
  that.
