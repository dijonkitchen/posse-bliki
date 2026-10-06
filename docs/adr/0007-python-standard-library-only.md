# 0007 — Python standard library only

- Date: 2026-10-06
- Status: Proposed (competes with a parallel proposal to rewrite the build in Rust)

## Context

The build had four runtime dependencies: markdown-it-py, Jinja2, PyYAML
and jsonschema. Each is a moving part with its own release cadence and
transitive dependencies, which cuts against the 10+ year horizon in
[ADR 0002](0002-no-quartz-no-vendored-ssg.md). Three of them do little
here: Jinja2 fills nine small templates, PyYAML parses five front-matter
keys, and jsonschema checks one small schema. Markdown is the only
genuinely hard part.

## Decision

The build uses only the Python standard library.

- `build/markdown.py` renders Markdown with the same algorithm as
  markdown-it (CommonMark, GFM tables, strikethrough, raw HTML escaped).
  Its output is byte-identical to the previous markdown-it-py
  `gfm-like` output on every note in this vault and in the Modin vault
  (773 files), and on 320,000 randomly generated inputs.
- `build/frontmatter.py` parses a strict YAML subset (flat mapping,
  scalars, flow and block lists) and validates it against
  `spec/content-schema.json` by hand. Unsupported YAML is a build
  error, not a silent misread.
- `build/render.py` replaces the Jinja templates with plain functions.
- `make serve` polls for changes instead of using watchfiles.

PyYAML and jsonschema move to the dev group: the harness keeps using them
as independent reference implementations to check every note's
front-matter.

## Consequences

- `python3 -m build` runs on any Python ≥ 3.13 with nothing installed.
  `uv` is still used for the test harness.
- The site output is unchanged byte-for-byte (verified by diffing the old
  and new builds).
- We now own a Markdown renderer (about 1,900 lines). Bugs in it are ours to fix,
  but behaviour is pinned by the harness and the vault itself.

## Alternatives considered

- **Keep markdown-it-py, drop the other three.** Smaller change, but keeps
  the one dependency most likely to change output between versions.
- **Rewrite in Rust with no crates.** A single static binary and no
  interpreter. The standard library lacks date formatting, JSON and an HTTP
  server, so those are hand-written too. Proposed separately for
  comparison.
