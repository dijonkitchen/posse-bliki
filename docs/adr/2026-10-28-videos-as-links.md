# 2026-10-28 — Videos are links, not embeds

- Date: 2026-10-28
- Status: Accepted
- Decided by: JC, asking to fix the YouTube embed on the "security" note,
  which showed as raw `<iframe>` code.

## Context

The build escapes raw HTML in notes, so an `<iframe>` pasted into a note
from the old Jekyll site rendered as literal text. Only one note had one:
a YouTube video in `notes/security.md`.

Turning the embed back on would mean letting raw HTML through the
Markdown renderer, loading YouTube's player and its tracking on every
read of the page, and adding a third-party frame to a site that
otherwise loads only its own files (ADR 2026-10-14-mobile-first; and,
if merged, the CSP in ADR 2026-10-28-csp-meta-tag would block it).

## Decision

- Videos are plain Markdown links to the video's page, with its title
  and where it's from.
- A harness test (`tests/harness/links.rs`) fails when a note's prose
  holds a raw HTML tag, since it would show as text. Code spans and
  fences, and autolinks like `<https://…>`, are fine.

## Consequences

- Readers click through to watch; nothing from YouTube loads until they
  do. The link works the same on GitHub, in Obsidian and on the site.
- Notes promoted from Modin, or pasted from elsewhere, that carry HTML
  fail the build until it's rewritten as Markdown.

## Alternatives

- **Allow raw HTML in Markdown.** It would open every note, and anything
  promoted into one, to arbitrary markup.
- **A click-to-load thumbnail.** Prettier, but it needs a hosted image per
  video and some markup or script; not worth it for one video.
