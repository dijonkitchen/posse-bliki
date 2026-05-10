# 0003 — uv for Python toolchain

- Date: 2026-05-10
- Status: Accepted

## Context

The build is Python. Python toolchain choices historically span pyenv,
virtualenv, pip, pip-tools, poetry, hatch, pipx — each solving a slice
of "install Python, manage a venv, lock dependencies, run a script."
Multiple tools means multiple things to keep working.

[uv](https://docs.astral.sh/uv/) collapses all of those into one binary.

## Decision

Use uv for everything: Python version management, dependency resolution,
lockfile, virtualenv, and script execution.

- `.python-version` pins the interpreter.
- `pyproject.toml` declares deps (PEP 621 standard).
- `uv.lock` is committed for reproducibility.
- All commands are `uv run …` — no manual venv activation.
- CI uses `astral-sh/setup-uv` and runs `uv sync` then `uv run pytest`.

## Consequences

- One tool to learn, install, and replace.
- Builds are fully reproducible: same Python version, same deps, same
  lockfile, same output (see [build-invariants spec](../../spec/build-invariants.md)).
- `uv run script.py` makes one-off utilities hermetic — they declare
  their own deps via PEP 723 inline metadata if needed.
- Cost: uv is young (~2 years at time of writing). If it disappears,
  we still have a standards-compliant `pyproject.toml`; pip, hatch, or
  poetry can take over with a `pip install -e .`. So the choice is
  reversible.

## Alternatives considered

- **pip + venv + requirements.txt.** Lowest common denominator, no lock
  semantics. Rejected — irreproducible builds are a build-invariants
  violation.
- **Poetry.** Mature, but slower, more opinionated, and its lockfile
  format is non-standard. uv is strictly simpler.
- **Hatch.** Closest competitor. Fine choice. uv wins on speed and
  on Python-version management being built in (Hatch defers to pyenv).
