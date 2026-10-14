# 2026-10-14 — Site and author naming

- Date: 2026-10-14
- Status: Accepted
- Decided by: JC, suggesting "JC (Jonathan Chen)" for the author and a
  title that matches the domain, "Dijon Kitchen", with napkin logic and
  "Philosophy as a service" in the subtitle.

## Context

The site was titled "Philosophy as a service", with the tagline "Bliki of
Jonathan Chen's thoughts." and the author shown as "Jonathan Chen".

- The domain is dijonkitchen.org. A title that differs from the domain makes
  people check twice that they are in the right place, in browser tabs,
  bookmarks, feed readers and search results.
- JC goes by JC. "Napkin logic" is what the Substack, which used to live at
  this domain, called his writing, so Substack readers know that phrase.
- The author name is also data. IndieWeb tools (microformats parsers,
  Bridgy, webmention.io, fediverse bridges) read the h-card `p-name` to show
  who wrote a post, and people search for the full name. "JC" alone is too
  short to find or to tell apart from other people.

## Decision

- **Title:** "Dijon Kitchen". It heads every page and ends each page's
  `<title>` ("Post — Dijon Kitchen"), so it is kept short.
- **Tagline:** "JC's napkin logic bliki. Philosophy as a service." It is the
  RSS and JSON Feed description.
- **Author:** readers see "JC (Jonathan Chen)" on the home page h-card and
  in each post byline. `author_label` in `build/site.rs` renders both. The
  markup is `<span class="p-nickname">JC</span> (<span class="p-name">Jonathan Chen</span>)`,
  so parsers read `name` as "Jonathan Chen" and `nickname` as "JC".
- Feeds keep "Jonathan Chen" as the author name. They have no nickname
  field, and the full name is the more useful one there.

## Consequences

- One new `Config` field, `author_nickname`.
- The byline link no longer carries `p-name` itself; the inner span does.
- Search results will show the new title as pages are recrawled. The domain
  and URLs don't change, so links and ranking carry over.

## Alternatives considered

- **Keep "Philosophy as a service" as the title.** No recrawl churn, but the
  title doesn't match the domain, and it is kept in the tagline anyway.
- **"Dijon Kitchen: JC's napkin logic bliki" as the full title.** Says it
  all, but appended to every page title it gets cut off in tabs and search
  results.
- **Author "JC" only.** Shortest, but parsers and search would see a
  two-letter name.
- **`p-name` "JC (Jonathan Chen)" as one string.** Simpler markup, but every
  consumer would show the parenthesis as part of the name.
