# 2026-10-16 — The home page shows recent posts

- Date: 2026-10-16
- Status: Proposed (JC to confirm; the option Claude recommended)
- Raised by: JC, who asked whether the home page needs the byline, the
  "Start here" list and the welcome text, or just recent posts.

## Context

The home page opened with a byline, three paragraphs of welcome, a
"Start here" list, and then the recent posts. Readers who come back for new
writing had to scroll past the same introduction every time. The old Jekyll
home page was just a list of posts.

## Decision

- The home page shows the site title and tagline, then the ten most recent
  posts, then a link to all notes. `content/index.md` has no body; if it
  gets one again, it renders above the posts.
- The welcome text (bliki, POSSE, notes evolve) and the "Start here" links
  move to a "New here?" section on the About page.
- The author h-card moves to the home page footer as "By JC". It stays on
  the home page because IndieWeb tools (Bridgy, IndieAuth) look for a
  representative h-card there.

## Consequences

- First-time readers find the introduction one click away, under About.
- The h-card's data is unchanged (ADR 2026-10-16-site-title-heads-home).

## Alternatives considered

- **Keep one line of welcome above the posts.** Gives first-time readers
  context, but repeats what the tagline already says.
- **Keep the page as it was.** The writing stays below the fold on phones.
