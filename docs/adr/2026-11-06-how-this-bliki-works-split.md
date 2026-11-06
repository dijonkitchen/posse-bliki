# 2026-11-06 — Split "How this bliki works" between the README and the colophon

- Date: 2026-11-06
- Status: Accepted
- Decided by: JC, asking why the note existed and whether it belonged in
  the colophon, a note of its own or the README.

## Context

`content/notes/how-this-bliki-works.md` came in with the first scaffold
([PR #1](https://github.com/dijonkitchen/posse-bliki/pull/1), 2026-05-10),
when the site was to be built with Quartz. It was a sample note showing
the authoring conventions: Obsidian, drafts, aliases, publishing with a
push, syndicating. No decision chose it and no ADR covers it. It stayed
through the rebuild (ADR 2026-05-10-spec-and-harness-over-implementation)
because `tests/harness/url_policy.rs` used it as the note that must build,
and later changes only kept its text current.

By November it was a poor fit in three ways:

- **It was a post.** Everything in `content/notes/` is a post
  (spec/content-schema.md), so a how-to about editing the repo showed up
  among the recent posts on the home page, in the RSS and JSON feeds and
  on a `meta` tag page of its own.
- **It was written for the author, but shown to readers.** Its steps
  (open `content/` in Obsidian, then `git push`) are for whoever edits
  the repo. About linked it to readers as "How to use this bliki".
- **It repeated other docs, and had drifted from them.** The README's
  "Daily workflow" and "POSSE and webmentions" sections and
  spec/content-schema.md already cover the same ground, and more exactly.
  The note said every deploy sends new posts to Bridgy, but the
  syndicate job only runs once the `SEND_WEBMENTIONS` variable is set
  (ADR 2026-10-11-webmentions-at-deploy-time). The only thing that wasn't
  written down anywhere else was the Obsidian setting that makes new links
  relative paths (ADR 2026-10-16-relative-markdown-links).

The colophon ("How this site is built and why") overlapped only on the
tool names in its table. About tells readers to see the colophon for how
syndication works, but the colophon only named the tools.

## Decision

Give each part of the note to the place its audience already looks, and
delete the note:

- **The README** gains the Obsidian link setting and one paragraph naming
  the front-matter fields an author uses day to day (`draft`, a future
  `date`, `aliases`, `syndication`), pointing to spec/content-schema.md
  for the full list. The spec stays the single source of truth for those
  fields.
- **The colophon** gains one short paragraph for readers on
  [POSSE](https://indieweb.org/POSSE): every post is published here
  first, copies elsewhere link back and are listed under the post, and
  replies can come back as
  [webmentions](https://www.w3.org/TR/webmention/). Its pointer to the
  note now points to the repo's README, linking the repo.
- **About** drops its "How to use this bliki" link. Its "How this site is
  built" link already goes to the colophon.
- **The old URL** `/notes/how-this-bliki-works/` becomes an alias on the
  colophon, because URLs are forever (spec/url-policy.md, after Tim
  Berners-Lee's [Cool URIs don't change](https://www.w3.org/Provider/Style/URI)).
  A harness test checks the redirect, and `url_policy.rs` checks a
  different note (`notes/bliki/`). This is the first alias inside
  `/notes/`, so the harness's list of posts now skips alias redirects,
  as its other checks already did. A renamed note's old slug would have
  hit the same problem.

## Consequences

- The feeds, the home page and the notes list stop showing a how-to as a
  post. The `meta` tag page goes away, because no other note used the tag.
- The authoring guide lives in one place, next to the code it describes,
  so it changes in the same PR as the behaviour it documents.
- The colophon stays short: one paragraph more, and no author steps.
- Feed readers that already have the item keep it. Their links to it
  redirect to the colophon.

## Alternatives considered

- **Keep it as a note and correct it.** It would still be a post in the
  feed, aimed at the author, repeating the README.
- **Turn it into a page rewritten for readers.** Reader-facing "how it
  works" is already split between About, Follow and the colophon. A
  fourth page would mostly repeat them.
- **Move all of it into the colophon.** That would put author steps
  (Obsidian settings, front matter) on a reader-facing page and make it
  long, when the colophon is meant to stay a short list of how and why.
