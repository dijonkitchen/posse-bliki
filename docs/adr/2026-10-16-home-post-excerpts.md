# 2026-10-16 — Home page posts show an excerpt

- Date: 2026-10-16
- Status: Accepted
- Decided by: JC, choosing "Excerpt" over links only or full notes.

## Context

The home page lists the ten latest posts (ADR 2026-10-16-home-shows-recent-posts)
as a title and a date. Many titles are one word ("raw", "meld", "assert"), so
the list says little about what each post is. The old Jekyll home page showed
an excerpt under each title.

## Decision

- Under each post's title and date, the home page shows the post's `summary`,
  or, without one, the start of its text up to 160 characters, cut at a word
  and ending with "…". It is the same text the page uses for its meta
  description, marked up as the h-entry's `p-summary`.
- `/notes/` and tag pages are unchanged: they list every post, so they stay
  compact.

## Consequences

- The home page is longer, about three lines per post on a phone.
- Adding a `summary` to a note's front matter now also sets its home page
  excerpt.

## Alternatives considered

- **Links only.** Shortest, but one-word titles give no hint of the post.
- **Full notes.** The home page becomes very long, and every note appears
  twice on the site.
