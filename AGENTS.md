# Operating contract for agents

You are working on a personal POSSE bliki. This file is read by every
agent and human who touches the repo. Read it fully before changing
anything.

## The contract

1. **Spec is the source of truth.** [`spec/`](spec/) defines what
   "correct" means. The build is regenerable; the spec is not.
2. **Tests enforce the spec.** [`tests/`](tests/) is an automated
   harness. A change is correct iff `uv run pytest` is green.
3. **Edit `build/` only.** Don't add new dependencies, frameworks, or
   abstractions outside this directory unless an ADR justifies it.
4. **Don't bypass the harness.** If a test fails, fix the implementation
   or update the spec + ADR — never silence the test.

## How to add a feature

1. Read relevant specs in `spec/`. Update them if the feature changes
   what "correct" means.
2. If the change is architectural (new dep, new layer, new external
   service), add an ADR in `docs/adr/` numbered after the latest one.
3. Add or update a test in `tests/` that fails today and passes when
   the feature is done.
4. Edit `build/` (and `build/templates/`) to make it pass.
5. Run `uv run pytest && uv run python -m build` locally. Both green.
6. Commit with a message that names the spec file or ADR number.

## How to fix a bug

Same loop, starting from step 3: write a regression test that reproduces
the bug, then fix it.

## Commands

```sh
uv sync                              # install / update deps
uv run pytest                        # run the harness
uv run pytest tests/test_foo.py -k bar   # focused
uv run python -m build               # build site → public/
uv run python -m build --serve       # build + serve at :8080 with watch
```

## Files you will touch often

- `content/` — the Obsidian vault. Edit notes here.
- `spec/` — outcome contracts. Edit when intent changes.
- `tests/` — harness. Edit when adding/fixing features.
- `build/` — implementation. Replaceable.

## Files you should rarely touch

- `docs/adr/` — append-only. New ADRs only.
- `pyproject.toml`, `uv.lock`, `.python-version` — change with an ADR.

## Hard rules

- No network access during build.
- No randomness, no `datetime.now()` in output.
- No new dependencies without an ADR.
- No unresolved wikilinks, no broken internal links — both are build errors.
- No `<script>` tags in output (the site is JS-free).
- No editing snapshot files in `tests/snapshots/` without a corresponding
  spec change explaining why output changed.
