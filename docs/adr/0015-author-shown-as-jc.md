# 0015 — Author shown as "JC (Jonathan Chen)"

- Date: 2026-10-14
- Status: Accepted
- Decided by: JC, suggesting "JC (Jonathan Chen) instead?"

## Context

The site showed the author as "Jonathan Chen" in the home page h-card and in
every post byline. JC goes by JC, and wanted that to be the name readers see.

The name is also data. IndieWeb tools (microformats parsers, Bridgy,
webmention.io, fediverse bridges) read the h-card's `p-name` to show who
wrote a post, and people search for the full name. "JC" alone is too short to
find or to tell apart from other people.

## Decision

- Readers see "JC (Jonathan Chen)" on the home page h-card and in each post
  byline. `author_label` in `build/site.rs` is the one place that renders it.
- The markup is `<span class="p-nickname">JC</span> (<span class="p-name">Jonathan Chen</span>)`,
  so parsers read `name` as "Jonathan Chen" and `nickname` as "JC".
- Feeds (RSS, JSON Feed) keep "Jonathan Chen" as the author name. They have
  no nickname field, and the full name is the more useful one there.
- The site title stays "Philosophy as a service".

## Consequences

- One new `Config` field, `author_nickname`.
- The byline link no longer carries `p-name` itself; the inner span does.
  The spec harness checks for `p-name` on the h-card and still passes.

## Alternatives considered

- **"JC" only.** Shortest, but parsers and search would see a two-letter
  name.
- **"Jonathan Chen" only** (what the site had). Accurate, but not the name
  JC goes by.
- **`p-name` "JC (Jonathan Chen)" as one string.** Simpler markup, but every
  consumer would show the parenthesis as part of the name.
