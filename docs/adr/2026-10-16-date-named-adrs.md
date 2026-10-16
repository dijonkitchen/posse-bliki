# 2026-10-16 — Every ADR named by date

- Date: 2026-10-16
- Status: Accepted
- Decided by: JC, asking to "refactor all ADRs to use dates".
- Amends: the naming rule in [`README.md`](README.md), which kept the
  numbered ADRs (0001–0016) under their numbers.

## Context

ADRs started out numbered (0001–0016). Parallel branches kept racing for
the next number, so since PR #14 new ADRs are named `YYYY-MM-DD-short-slug.md`
and the numbered ones were left alone. That left two schemes side by side:
an index that sorted numbers above dates, references written as `ADR 0007`
next to `ADR 2026-10-14-www-as-canonical-host`, and gaps in the numbers
(0006, 0012–0015) that look like missing files.

## Decision

- Every ADR is named `YYYY-MM-DD-short-slug.md`. The eleven numbered ones
  are renamed with `git mv`, keeping their slug, and dated with the `Date:`
  each one already states (the day it was accepted, not the day it merged).
- Each heading reads `# YYYY-MM-DD — Title`, as the date-named ones do.
- Every reference moves to the new name: specs, code and test comments,
  workflows, `Cargo.toml`, `README.md`, `AGENTS.md` and the ADRs themselves.
  A reference is the file name stem, e.g. `ADR 2026-10-06-rust-build-zero-dependencies`.
- The index lists ADRs oldest first, by date.
- This is the one exception to "append-only": the files are moved and their
  headings and cross-links updated, but no Context, Decision, Consequences or
  Alternatives text changes.

## Consequences

- One naming scheme. File listings and the index sort in the order the
  decisions were made.
- Old commit messages and merged PR bodies still say `ADR 0007` and so on.
  `git log --follow docs/adr/<new name>.md` shows the file's old name, and
  the table below maps each number to its new name:

  | Number | Now |
  |---|---|
  | 0001 | 2026-05-10-spec-and-harness-over-implementation |
  | 0002 | 2026-05-10-no-quartz-no-vendored-ssg |
  | 0003 | 2026-05-10-uv-for-python-toolchain |
  | 0004 | 2026-05-10-indieweb-standards-supported |
  | 0005 | 2026-05-10-obsidian-as-editor-no-required-plugins |
  | 0007 | 2026-10-06-rust-build-zero-dependencies |
  | 0008 | 2026-10-11-webmentions-at-deploy-time |
  | 0009 | 2026-10-11-a11y-design-system-self-hosted-fonts |
  | 0010 | 2026-10-11-rust-only-toolchain |
  | 0011 | 2026-10-11-holocene-years-on-the-page |
  | 0016 | 2026-10-14-supply-chain-pinned-actions |

- References are longer. A stem is unambiguous where a date alone is not
  (four ADRs share 2026-05-10), so the length is the price of that.
- A branch opened before this change that adds a numbered ADR or links to
  one by its old path renames on its next merge from main.

## Alternatives considered

- **Keep the numbers on old ADRs** (the rule until now). No churn, but two
  schemes forever, and every new reader asks why.
- **Date by first commit or merge instead of the stated `Date:`.** Git dates
  are when the file landed, which for 0008–0011 is a day off the decision
  and for 0010 three days off. The stated date is what each ADR claims.
- **Date only, no slug** (`2026-05-10.md`). Collides when several ADRs share
  a day.
- **Leave a stub at each old path pointing to the new one.** Keeps old links
  working, but doubles the folder for links that only commit messages hold,
  and `git log --follow` already answers that.
