# 2026-10-16 — No hand-kept ADR index

- Date: 2026-10-16
- Status: Proposed (JC to confirm; the option Claude recommended)
- Raised by: JC, who asked whether the ADR list is worth having after
  PRs #24 and #32 conflicted on it.

## Context

`docs/adr/README.md` held a table of every ADR with its date, title and
status. Every new ADR added a row at the end of it, so any two open PRs that
each added an ADR conflicted there, as #24 and #32 did.

ADR file names start with the date (ADR 2026-10-16-date-named-adrs), so the
folder listing on GitHub and in an editor already shows them in order. Each
ADR has its title and status at the top.

## Decision

- Remove the table. The README keeps the rules for naming and format.
- A superseded ADR's status line links the ADR that replaces it. This is
  the only edit an accepted ADR gets. The two existing cases
  (2026-05-10-uv-for-python-toolchain and 2026-10-14-site-and-author-naming)
  now say so in their own status lines.

## Consequences

- Adding an ADR touches only its own file, so parallel PRs don't conflict.
- Readers see titles by opening files rather than in one table.

## Alternatives considered

- **Generate the table** from each ADR's title and status at build time,
  with a test that it is current. No conflicts, but it is code to maintain
  for a list the folder already gives.
- **Keep the hand-kept table** and fix the conflict each time.
