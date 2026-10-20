# Build invariants

Properties of the build process itself. Each enforced by `tests/`.

## Hermetic

- The build does not read any file outside the repository.
- The build does not make network calls.
- All versions are pinned: the Rust compiler in `rust-toolchain.toml`, the
  edition in `Cargo.toml`. Dependencies (build, tests or tools) and
  external assets: none. Why: the same input must give the same output on
  every machine, so nothing that varies between machines may feed the
  build (ADR 2026-10-06-rust-build-zero-dependencies, ADR 2026-10-11-rust-only-toolchain).

Tested by running the build with no network and verifying success.

## Supply chain

- `Cargo.lock` lists exactly one package, this one (ADR 2026-10-06-rust-build-zero-dependencies). Cargo runs
  with `--locked`, so a change that would add or update a crate fails.
- Every third-party GitHub Action is pinned to a full commit SHA, not a
  tag, because a tag can be moved to new code after review (ADR 2026-10-14-supply-chain-pinned-actions).
- Every job names an OS release for its runner (`ubuntu-24.04`), never a
  moving `-latest` label.
- Workflows grant the `GITHUB_TOKEN` the least it needs per job; the job
  that runs repository code cannot publish the site.

Tested by `tests/harness/supply_chain.rs` (all but the last).

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
- The clock is read once, for the publish cutoff: notes dated after today
  (UTC) are left out (ADR 2026-10-20-scheduled-posts). `--today YYYY-MM-DD`
  replaces the clock, so a build is still a function of its inputs, and
  tests pin it.
- No randomness. No PIDs. No hash-of-iteration-order in output.

## Fast enough

Cold build of a vault with 1000 posts completes in under 5 seconds on a
laptop. Not currently tested (not the bottleneck), but a non-goal to break.

## Fail loudly

The build exits non-zero on:

- Front-matter that doesn't validate against `spec/content-schema.json`.
- Two notes with the same filename stem (wikilink resolution would be ambiguous).
- An unresolved wikilink.
- A relative `.md` link to a file that isn't a published note.
- A wikilink to a draft.
- A spec-violating output (caught by post-build harness).

There is no "warn and continue" path. CI is the only place builds happen
for production, and CI must be green.
