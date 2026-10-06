# 0007 — Rust build with zero dependencies

- Date: 2026-10-06
- Status: Proposed

## Context

[ADR 0001](0001-spec-and-harness-over-implementation.md) makes the build
replaceable: `spec/` and `tests/` are the durable artifacts. The first
implementation was ~530 lines of Python standing on four runtime
dependencies — jinja2, pyyaml, jsonschema and markdown-it-py (plus their
transitive deps: markupsafe, mdurl, mdit-py-plugins, referencing,
rpds-py, attrs). That is a lot of moving parts under a site meant to last
ten years, and every one of them is a supply-chain and bit-rot risk for
output that must stay byte-stable.

This ADR competes with a parallel proposal to keep Python but drop to the
standard library only. Both remove the dependencies; they differ in what
the build is written in.

## Decision

Reimplement the build as a Rust program using the standard library and
nothing else.

- `Cargo.toml` at the repo root has an empty `[dependencies]` section and
  one binary, `bliki`, at `build/main.rs`. `Cargo.lock` is committed and
  contains exactly one package: this one.
- The former templates are plain Rust functions in `build/site.rs` that
  reproduce Jinja's autoescape semantics (`& < > " '`).
- `build/markdown.rs` is a port of markdown-it-py's `gfm-like` preset with
  `html=False, linkify=False, typographer=False`, so Markdown output is
  byte-identical to the previous build. `build/mdurl.rs` ports the URL
  normalisation (including punycode) that markdown-it applies to links.
- `build/frontmatter.rs` parses the YAML subset the vault actually uses
  (scalars, quoted and block scalars, flow and block sequences; dates stay
  strings) and hand-checks `spec/content-schema.json`.
- `build/date.rs` does the calendar maths for RFC 822 and RFC 3339 dates;
  `build/json.rs` emits JSON byte-identical to Python's
  `json.dumps(indent=2)`, `ensure_ascii` included.
- `build/entities.rs` and `build/unicode.rs` are generated tables (HTML5
  named references; Unicode punctuation/word classes) so character classes
  match the reference implementation exactly.
- The CLI keeps the old interface: `bliki --content DIR --out DIR`, plus
  `--print-config` and `--serve --port N` (a `std::net::TcpListener` server
  that polls content mtimes and rebuilds).
- Python stays, as the test harness only: `uv run pytest` drives the binary
  as a subprocess. `pyproject.toml` has `dependencies = []`; pyyaml and
  jsonschema move to the dev group, where `tests/test_frontmatter_schema.py`
  uses them as an *independent* check of the vault's front-matter.

Byte-for-byte equivalence with the previous Python build was verified on the
whole vault (`diff -r`), and the Markdown renderer was differentially fuzzed
against markdown-it-py over tens of thousands of generated documents.

## Consequences

- Zero build dependencies: nothing to audit, pin, or chase for CVEs. The
  lockfile cannot drift because there is nothing in it.
- One static binary, no interpreter or venv at build time. CI gets simpler:
  `ubuntu-latest` and the GitHub Pages runner already ship cargo.
- Fast: a cold build of this vault is ~15ms, so the "1000 posts in under 5
  seconds" invariant has a lot of headroom.
- Cost: more code than the Python version (~5k lines vs ~530), most of it the
  Markdown parser and the generated Unicode tables. That code is a port of a
  specified format rather than business logic, and the harness pins its
  behaviour, but it is still code we now own.
- Cost: changing the build now needs a Rust toolchain. Contributors who only
  edit `content/` need nothing new.
- Cost: the generated tables (`entities.rs`, `unicode.rs`) are tied to a
  Unicode version. They are data, regenerable, and only affect edge-case
  character classification.
- Reversible: the harness defines correctness, so a future ADR can replace
  this implementation with anything that keeps `uv run pytest` green.

## Alternatives considered

- **Python, standard library only** (the parallel proposal). Keeps one
  language in the repo and is much less code, since the harness is already
  Python. Loses the single-binary deployment and is ~100× slower, and it
  still carries an interpreter and a venv into the build step. A reasonable
  choice; this ADR prefers a build artifact with no runtime at all.
- **Keep the four Python dependencies.** Simplest to maintain today, and the
  libraries are good ones. Rejected because the dependency surface is the
  main long-term risk to byte-stable output, which is exactly what
  [build-invariants](../../spec/build-invariants.md) promises.
- **Rust with crates** (`pulldown-cmark`, `serde_yaml`, `tera`, …). Far less
  code, but `pulldown-cmark` does not reproduce markdown-it's output, so
  existing pages would change; and it reintroduces the dependency tree this
  ADR exists to remove.
- **An off-the-shelf SSG** (Hugo, Zola, Quartz). Already rejected by
  [ADR 0002](0002-no-quartz-no-vendored-ssg.md); unchanged here.
