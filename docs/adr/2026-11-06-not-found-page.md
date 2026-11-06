# 2026-11-06 — A 404 page that helps lost readers on

- Date: 2026-11-06
- Status: Accepted
- Decided by: JC, asking to make sure the site has a 404 page.

## Context

The build already wrote `/404.html`, and GitHub Pages already served it
with a 404 status for any address that has no page. It was thin: one
line linking home and the notes, with the site tagline as its
description and nothing telling search engines to skip it. Pages also
serves the file at `/404.html` itself with a 200, so that copy could be
indexed as a page of the site.

How Pages and the site treat the page shapes what it can do:

- Pages serves the same file at every depth (`/nope`,
  `/notes/old/post/`), so a relative link in it would point somewhere
  different from each address. Every link the shared template emits is
  already root-relative, and the sub-path build prefixes them
  (spec/output-contract.md).
- There's no server code to read the missing address, and the page has
  to work with JS off (ADR 2026-10-18-style-picker), so it can't name
  the address or suggest a close match.
- Moved pages already keep their old address through aliases and
  redirects, which are never removed (spec/url-policy.md), so a reader
  here has a typo or a broken link, not a page that moved.
- The offline worker keeps 200 responses only
  (ADR 2026-10-29-offline-reading), so a 404 is shown online but never
  kept.

## Decision

- **Say what happened, then offer three ways on**: the home page for
  the latest posts, all notes newest first, and the follow page for a
  feed reader (when the vault has one, like the footer link;
  ADR 2026-11-05-follow-with-a-feed-reader). The header, style picker
  and footer stay the same as on every page, so it looks like the site
  in every style.
- **Title and heading "Page not found"**, plain words a screen reader
  announces first, in the page's only `<h1>`.
- **`noindex`, no canonical, not in the sitemap**, like `/offline.html`.
  It has no description, since only indexable pages need one
  (spec/performance.md).
- **The worker stays as it is.** Online, a missing address shows this
  page; offline, it shows the offline page, because the worker can't
  tell a missing page from one that isn't saved. Not keeping 404s means
  a post published at an address later is never answered offline by an
  old "not found".
- **No Lighthouse SEO budget for `noindex` pages.** A PR that changes
  the 404 gets it audited (ADR 2026-10-29-lighthouse-changed-pages), and
  Lighthouse's SEO category scores any `noindex` page 54: it fails
  "Page is blocked from indexing", and "no meta description" with it.
  Both are the point of the page, so `scripts/lighthouse.sh` holds a
  page carrying `noindex` to the performance, accessibility and best
  practices budgets only, the same line the harness draws for
  descriptions (spec/performance.md). The offline page and alias
  redirects get the same treatment the next time a PR changes them.
- `tests/harness/not_found.rs` checks the links work at any depth, the
  page links home, the notes and follow, it stays out of search, and
  the worker keeps 200s only.

## Consequences

- A reader who lands on a dead link has somewhere to go without the
  back button, and in the style they picked.
- `/404.html` drops out of search results.
- The page can't say which address was missing or suggest a fix; doing
  that would take JavaScript or a server.

## Alternatives considered

- **Leave it as it was.** It worked, but offered less to a lost reader
  and could be indexed at `/404.html`.
- **A search box.** The site has no search, and an external one would
  break the CSP (ADR 2026-10-28-csp-meta-tag) and send readers' queries
  elsewhere.
- **Suggest a close match with JavaScript.** It would need the list of
  every page shipped to the reader and would do nothing with JS off.
- **Leave the 404 indexable** to keep its Lighthouse SEO score at 100.
  Crawlers skip it at missing addresses anyway (404 status), but the
  copy at `/404.html` answers 200 and could show up in results.
- **Keep the 404 in the worker's shell** so offline readers see "not
  found" for missing pages. The worker can't know an unsaved page is
  missing, so it would show "not found" for pages that exist.
