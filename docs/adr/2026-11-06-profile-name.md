# 2026-11-06 — "JC (Jonathan Chen)" as the name other sites show

- Date: 2026-11-06
- Status: Accepted
- Decided by: JC, asking "how about JC (Jonathan Chen)?"
  after asking whether the profile should say JC instead of the full
  name.

## Context

Readers on the site see "JC": the "By JC" bylines and the home page's
h-card (ADR 2026-10-16-site-title-heads-home).
The h-card's `p-name` was the full name, "Jonathan Chen",
in a `<data>` element, with "JC" as `p-nickname`
(ADR 2026-10-14-site-and-author-naming).
The JSON Feed author and the blogroll OPML's owner were "Jonathan Chen"
too.

Other sites don't show the page, they show `p-name`:
feed readers, webmention displays and Bridgy Fed's fediverse profile
all name the author from it.
So everywhere off the site JC was "Jonathan Chen",
a name JC doesn't go by on the site.

ADR 2026-10-14-site-and-author-naming turned down "JC (Jonathan Chen)"
as one `p-name` string because every consumer would show the
parenthesis. That is now the point: JC's GitHub name is
"JC (Jonathan Chen)", and it's the name JC wants other sites to show.

## Decision

- `p-name` on the home h-card and every post byline is
  "JC (Jonathan Chen)": the nickname, then the full name in parentheses.
  The build composes it from `author_nickname` and `author_name`
  (`author_display_name` in `build/site.rs`),
  so the two can't drift apart.
- The JSON Feed's author name and the blogroll OPML's `ownerName`
  use the same name.
- The page still shows "By JC"; `p-nickname` stays "JC".
- The config's JSON exports `author.nickname` next to `author.name`,
  and the microformats tests expect `p-name` to be
  "nickname (name)".

## Consequences

- Readers, webmention displays and Bridgy Fed show "JC (Jonathan Chen)",
  matching GitHub, and searches for the full name still match.
  Bridgy Fed updates when JC presses "Update profile" there.
- Parsers that split `p-name` into given and family names get it wrong;
  none of the tools the site works with do.

## Alternatives considered

- **"Jonathan Chen" (as before).** The name work contacts search for,
  but not the one JC uses on the site.
- **"JC" only.** Matches the page, but a two-letter name is hard to
  find or recognise off the site.
- **"JC" plus hidden `p-given-name` and `p-family-name`.** Keeps the
  full name in the markup, but tools display `p-name`, so readers would
  only see "JC".
