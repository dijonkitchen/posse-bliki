# 2026-10-16 — Lighthouse budget scored by the median of three runs

- Date: 2026-10-16
- Status: Accepted
- Decided by: JC, who said yes to auditing each page three times and using
  the median after a one-off CI failure on PR #24.

## Context

The Lighthouse job (ADR 2026-10-14-mobile-first-readable-output-lighthouse)
ran one audit per page. On PR #24 the home page scored performance 88, under
the budget of 90. The only failing audit was Total Blocking Time, 480 ms, on
a page with no JavaScript. The home page is the first URL audited, while
the shared runner's Chrome is still warming up. The same commit scored 100
locally twice and passed when the job was re-run. One slow moment on the
runner failed the build, and nothing about the page had changed.

Lighthouse's own guidance on variability is to run several times and take
the median.

## Decision

- `scripts/lighthouse.sh` audits each page three times (`RUNS`, default 3).
- Each category is scored by its median run. When a median misses the
  budget, the job prints that run's failing audits, as before.
- The log line for each page shows all three scores, e.g.
  `performance 100 (88/100/100)`, so noise is visible without failing CI.
- The budgets are unchanged.

## Consequences

- The job takes about three times as long, roughly 3 minutes instead of 1.
  It runs in parallel with `test` and `html`, so a PR waits little longer.
- A real regression still fails: it has to show in at least two of three
  runs, and a slower page does.
- Every run's report is kept in the `lighthouse-reports` artifact.

## Alternatives considered

- **Lower the performance budget.** Hides real regressions as well as noise.
- **Warm-up run, then one audit.** Fixes the cold-start case only.
- **Retry once on failure.** Same cost when it fails, and a retry that
  passes says less than a median of three.
