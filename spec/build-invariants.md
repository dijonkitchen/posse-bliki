# Build invariants

Properties of the build process itself. Each enforced by `tests/`.

## Hermetic

- The build does not read any file outside the repository.
- The build does not make network calls.
- All versions are pinned: the Rust compiler in `rust-toolchain.toml`, the
  edition in `Cargo.toml`. Dependencies (build, tests or tools) and
  external assets: none. Why: the same input must give the same output on
  every machine, so nothing that varies between machines may feed the
  build (ADR 0007, ADR 0010).

Tested by running the build with no network and verifying success.

## Supply chain

- `Cargo.lock` lists exactly one package, this one (ADR 0007). Cargo runs
  with `--locked`, so a change that would add or update a crate fails.
- Every third-party GitHub Action is pinned to a full commit SHA, not a
  tag, because a tag can be moved to new code after review (ADR 0016).
- Workflows grant the `GITHUB_TOKEN` the least it needs per job; the job
  that runs repository code cannot publish the site.

Tested by `tests/harness/supply_chain.rs` (the first two).

## Idempotent

Running the build (`cargo run --release`) twice in a row produces
byte-identical output in `public/`. Tested by comparing every file from two
consecutive builds.

## Deterministic

- File walks are sorted before processing.
- Maps are never iterated for output; keyed collections are kept in sorted
  order (or as explicit ordered vectors).
- No wall-clock timestamps end up in output. The only time-like
  values are derived from front-matter `date` / `updated`.
- No randomness. No PIDs. No hash-of-iteration-order in output.

## Fast enough

Cold build of a vault with 1000 posts completes in under 5 seconds on a
laptop. Not currently tested (not the bottleneck), but a non-goal to break.

## Fail loudly

The build exits non-zero on:

- Front-matter that doesn't validate against `spec/content-schema.json`.
- Two notes with the same filename stem (wikilink resolution would be ambiguous).
- An unresolved wikilink.
- A wikilink to a draft.
- A spec-violating output (caught by post-build harness).

There is no "warn and continue" path. CI is the only place builds happen
for production, and CI must be green.
