# 2026-10-29 — Lighthouse also audits the pages a PR changes

- Date: 2026-10-29
- Status: Accepted
- Decided by: JC, who asked whether Lighthouse should check the pages a PR
  changes.

## Context

CI audited the same six pages on every run: home, a post with code, a post
with a large photo, a post with a video embed, `/notes/` and `/about/`
(ADR 2026-10-27-parallel-lighthouse). They cover every kind of page, so a
template or CSS change is caught. A PR that adds or edits any other post
was never audited itself, so a heavy image or a missing alt text on that
post could merge unseen. The list also lived in two places, the script and
the CI matrix.

## Decision

- `scripts/lighthouse-pages.sh` owns the list. With no argument it prints
  the six fixed pages; `scripts/lighthouse.sh` uses that as its default, so
  `make lighthouse` is unchanged.
- Given a base ref, it also builds the site at that ref and at HEAD, with
  the same `--today`, and adds every built HTML page that is new or
  differs. Diffing the output rather than the changed files catches pages
  touched indirectly, such as a post whose linked note changed title.
- More than 10 changed pages means a site-wide change (a template, the
  header). Then it adds none, since the fixed pages already stand for every
  kind of page and auditing all 160-odd would be slow.
- A new `lighthouse-pages` job runs the script with the PR's base commit
  and feeds the list to the `lighthouse-page` matrix. Pushes to `main` get
  just the fixed pages, which the PR already audited.

## Consequences

- A PR's new and edited posts are held to the same budget as the fixed
  pages.
- The `lighthouse-pages` job builds the site twice before the audits start,
  about 25 s more wall time. Each added page is one more parallel job, so
  it barely adds wall time.
- The page list lives in one place.
- CSS-only changes leave the HTML alone, so they get the fixed pages, which
  is what they need.

## Alternatives considered

- **Map changed `content/` files to pages.** No second build, but misses
  pages a template or a linked note changes, and needs its own copy of the
  build's URL rules.
- **Audit every page on every PR.** Full coverage, but about 160 jobs a
  run for changes that almost never affect more than a few pages.
- **Random pages each run.** Spreads coverage over time, but a failure
  wouldn't reproduce on a re-run and wouldn't relate to the PR.
