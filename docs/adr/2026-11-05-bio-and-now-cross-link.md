# 2026-11-05 — The bio and the now page link to each other

- Date: 2026-11-05
- Status: Accepted
- Decided by: JC, asking to "cross link resume in now page when ready".

## Context

The now page (ADR 2026-11-01-now-page) says what JC is doing at the
moment; the bio (ADR 2026-10-16-bio-page) is the steadier summary of past
work. Someone reading one often wants the other: a recruiter on the bio
wants to know what JC is up to now, and a reader of Now who sees JC's job
may want the history behind it.

## Decision

- The two stay separate pages, since they change at different rates.
- The now page's Work section and the About page link to the bio, and
  the bio's "More about me" section links to the now page.
- On About the link is in the opening paragraph, since visitors often
  expect "About" to be about the person, and here it's about the site
  (JC agreed 2026-11-05).
- About keeps its email riddle, "contact me at `${my-initials}` [at]
  MY_DOMAIN_NAME", and the bio uses the same one, so the two pages don't
  seem to give different addresses (JC asked to stick with that wording,
  2026-11-05).

## Consequences

- Each page is one tap from the other. The build fails on a link to a
  missing page, so deleting either page without its links is caught.
- The harness test that builds the site without a now page repoints
  links to it first, since the bio links there.

## Alternatives considered

- **Merge them into one page.** Fewer pages, but the dated now snapshot
  would churn the bio, and `/now/` would lose its conventional shape.
