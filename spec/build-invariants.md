# Build invariants

Properties of the build process itself. Each enforced by `tests/`.

## Hermetic

- The build does not read any file outside the repository.
- The build does not make network calls.
- All versions are pinned: Python in `.python-version`, deps in `uv.lock`,
  Quartz/external assets — none.

Tested by running the build with no network and verifying success.

## Idempotent

Running `build/build.py` twice in a row produces byte-identical output
in `public/`. Tested by hashing every file from two consecutive builds.

## Deterministic

- File walks are sorted before processing.
- Dict iteration is insertion-order (Python 3.7+ guarantee), and we always
  insert in sorted order.
- No timestamps from `datetime.now()` end up in output. The only time-like
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
