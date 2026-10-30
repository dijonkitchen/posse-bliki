# 2026-10-29 — Retry a Lighthouse run when Chrome fails to start

- Date: 2026-10-29
- Status: Accepted
- Decided by: JC, who asked to look into the Chrome error on PR #61.

## Context

On PR #61 the `/notes/governance/` Lighthouse job failed before auditing
anything:

```
Runtime error encountered: waiting for dynamic debugging port in chrome-err.log
```

Lighthouse's `chrome-launcher` starts Chrome and polls for its debugging
port for about 25 seconds. On that runner Chrome was still starting when
the wait ran out; the job's cleanup found it running. Nothing in the PR
caused it, and the other five pages passed. Since
ADR 2026-10-27-parallel-lighthouse every page's first run is on a cold
Chrome, so this can hit any page job. The wait isn't a Lighthouse CLI
option.

## Decision

`scripts/lighthouse.sh` retries a Lighthouse run that exits with an error,
up to 3 tries (`TRIES`), with a CI warning on each retry. Lighthouse exits
with an error only when it can't produce a report; a low score is still a
report, so budgets are judged exactly as before and a retry never hides a
failing score.

## Consequences

- A slow Chrome start costs one more try, about 30 seconds, instead of a
  red check and a manual re-run.
- A real launch failure (say, a broken Chrome on the runner image) still
  fails, after 3 tries.
- Retries show up as warnings, so a rise in them is visible.

## Alternatives considered

- **Re-run failed jobs by hand.** What happened on #61; it works, but
  every flake blocks a PR until someone notices.
- **Warm Chrome up before the first run.** Helps the cold start, but
  doesn't cover a slow start later in the job and adds a step that can
  itself flake.
- **Drive Lighthouse from Node with a longer launch wait.** Fixes the
  cause, but replaces the CLI with code to maintain for a rare flake.
