# 2026-11-05 — The now page and About link to the bio

- Date: 2026-11-05
- Status: Accepted
- Decided by: JC, asking to "cross link resume in now page when ready",
  and choosing on 2026-11-06 to ship the links with the bio in one PR,
  without a link back from the bio to the now page.

## Context

The bio (ADR 2026-10-16-bio-page) isn't in the header nav, so readers
need another way to reach it. The now page (ADR 2026-11-01-now-page)
names JC's current job, and a reader who sees it may want the history
behind it. Visitors often expect "About" to be about the person, and
here it's about the site.

## Decision

- The now page and the bio stay separate pages, since they change at
  different rates.
- The now page's Work section links to the bio.
- About links to the bio in its opening paragraph (JC agreed 2026-11-05).
- The bio doesn't link back to the now page. The now page is optional:
  the build links it only when the vault has one (ADR
  2026-11-05-now-in-the-header), and a harness test builds the site
  without it. A link from the bio would be a broken link in that build,
  so the test would have to rewrite it first.
- About keeps its email riddle, "contact me at `${my-initials}` [at]
  MY_DOMAIN_NAME", and the bio uses the same one, so the two pages don't
  seem to give different addresses (JC asked to stick with that wording,
  2026-11-05).
- The links ship in the same PR as the bio, since they're part of
  publishing it.

## Consequences

- The bio is one tap from the now page and from About. Recruiters reach
  the now page from the bio through the header, as on every page.
- No harness test changes.

## Alternatives considered

- **Merge the bio and the now page.** Fewer pages, but the dated now
  snapshot would churn the bio, and `/now/` would lose its conventional
  shape.
- **Link back from the bio to the now page.** Needs the no-now-page test
  to rewrite links, or that test and the follow and blogroll ones to
  build a small vault instead. Simpler to leave it out.
- **A separate PR for the links.** Keeps one change per PR, but stacked
  on the bio it was harder to follow than the change was worth.
