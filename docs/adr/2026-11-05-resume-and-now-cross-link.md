# 2026-11-05 — The résumé and the now page link to each other

- Date: 2026-11-05
- Status: Accepted
- Decided by: JC, asking to "cross link resume in now page when ready".

## Context

The now page (ADR 2026-11-01-now-page) says what JC is doing at the
moment; the résumé (ADR 2026-10-16-resume-page) is the steadier record of
past work. Someone reading one often wants the other: a recruiter on the
résumé wants to know what JC is up to now, and a reader of Now who sees
JC's job may want the history behind it.

## Decision

- The two stay separate pages, since they change at different rates.
- The now page's Work section and the About page link to the résumé, and the résumé's
  "More about me" section links to the now page.

## Consequences

- Each page is one tap from the other. The now page's link has to stay
  when Work is rewritten; the build fails on a link to a missing page,
  so deleting either page without its link is caught.

## Alternatives considered

- **Merge them into one page.** Fewer pages, but the dated now snapshot
  would churn the résumé, and `/now/` would lose its conventional shape.
