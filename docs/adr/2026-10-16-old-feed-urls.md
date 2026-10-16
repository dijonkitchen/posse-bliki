# 2026-10-16 — Keep the old Substack and Jekyll feed URLs

- Date: 2026-10-16
- Status: Accepted
- Decided by: found in a site consistency check JC asked for; JC reviews
  it in the PR.

## Context

Before the cutover, www.dijonkitchen.org was served by two earlier sites,
each with its own RSS feed:

- Substack, at `/feed`.
- The Jekyll site, at `/feed.xml`.

The new site's feed is at `/index.xml`, and both old URLs returned 404
after the cutover. Feed readers don't follow HTML redirects, so the alias
pages that keep old post links working (`/p/<slug>/`, `/<slug>/`) don't
help them: anyone subscribed through one of the old URLs silently stopped
getting posts.

GitHub Pages can't send an HTTP redirect for a single path.

## Decision

The build writes the RSS feed three times, byte for byte: `/index.xml`,
`/feed` and `/feed.xml`. All three name `/index.xml` as their
`atom:link rel="self"`, so readers that honour `self` move subscribers to
the canonical URL. Like aliases, the old URLs are never removed.

## Consequences

- Old subscribers get new posts again, and item guids are the same in all
  three, so a reader that ends up with two of them shows no duplicates.
- Pages serves `/feed` (no extension) as `application/octet-stream`.
  Feed readers read the body rather than trusting the type, so this works
  in practice, but a strict client could refuse it.
- A feed validator pointed at `/feed` or `/feed.xml` warns that `self`
  doesn't match the address it fetched. That's the intent.

## Alternatives considered

- **Leave them 404.** Simple, but loses every old subscriber without
  telling them.
- **An HTML redirect page at `/feed/`.** Readers don't follow it, and
  Pages would answer `/feed` with a 301 to `/feed/` that still ends in HTML.
- **A different host or a CDN rule for real 301s.** A new service for two
  URLs; the site stays on plain GitHub Pages.
