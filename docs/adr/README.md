# Architecture Decision Records

Each ADR captures one decision: context, options considered, the choice made,
and consequences. ADRs are **append-only** — to change a decision, write a new
ADR that supersedes the old one. Don't edit history.

The ADRs are the files in this folder. Their date-first names sort them in
the order they were decided, and each one states its status at the top,
including which later ADR supersedes it. There is no hand-kept index, so
two PRs that each add an ADR don't conflict
(ADR 2026-10-16-no-adr-index).

## Naming

New ADRs are named by date, `YYYY-MM-DD-short-slug.md`, so parallel branches
don't race for the next number. If two land on the same day, the slug keeps
them apart. The date is the day the decision was accepted. The ADRs that
were once numbered (0001–0016) were renamed to this form; see
[ADR 2026-10-16-date-named-adrs](2026-10-16-date-named-adrs.md) for the
mapping. Refer to an ADR by its file name stem,
e.g. `ADR 2026-10-14-www-as-canonical-host`.

## Format

Short. One page each. The four sections are: **Context**, **Decision**,
**Consequences**, **Alternatives considered**. Date and status at the top.
When a new ADR supersedes an old one, update the old one's status line to
link it; that line is the only edit an accepted ADR gets.
