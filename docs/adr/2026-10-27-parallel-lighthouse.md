# 2026-10-27 — Lighthouse audits one page per CI job, in parallel

- Date: 2026-10-27
- Status: Accepted
- Decided by: JC, who asked for faster CI.

## Context

CI runs three jobs side by side: `test` (about 40 s), `html` (about 25 s)
and `lighthouse` (about 4 minutes). A PR waits for the slowest, so
Lighthouse is almost all of CI's wall time. Of its 4 minutes, about 3.5 are
the audits themselves: six pages, three runs each
(ADR 2026-10-16-lighthouse-median-of-three), one after another at about
11 s a run. Setup (Rust toolchain, `npm ci`, building the site) is about
30 s.

## Decision

- The `lighthouse-page` job is a matrix with one job per audited page. Each
  sets `PAGES` to its page; `scripts/lighthouse.sh` still defaults to all
  six, so `make lighthouse` locally is unchanged.
- Each page is still audited three times and scored by the median, with the
  same budgets.
- A small `lighthouse` job waits for the matrix and passes only if every
  page passed, so the check name branch protection sees stays `lighthouse`.
- `fail-fast: false`, so one failing page doesn't hide the others' scores.
- Reports are uploaded per page as `lighthouse-reports-<n>`.

## Consequences

- Lighthouse takes about 1 minute of wall time instead of 4, so CI as a
  whole finishes in about a minute and a half.
- Each page pays the setup again: about 3 more minutes of runner time per
  run in total. The repository is public, so Actions minutes are free.
- Every page's first audit now runs on a cold Chrome, not just the home
  page's. The median of three already absorbs one slow run.
- The page list lives in two places, the script and the matrix. Comments
  in both point at each other.

## Alternatives considered

- **Audit pages concurrently in one job.** No extra setup, but several
  Chromes on one shared runner would make performance scores noisy, the
  problem the median of three exists to fix.
- **Fewer runs or fewer pages.** Faster, but gives up coverage or the noise
  protection.
- **Cache Cargo builds.** The build is about 12 s and restoring a cache
  costs a few seconds, so it would barely move the 4 minutes; it also adds
  an action and a cache another branch could poison.
- **Skip Lighthouse when only docs change.** A required check that never
  runs blocks the merge, and the saving only applies to some PRs.
